use std::{thread, time::Duration};
use thread::JoinHandle;

use anyhow::{Result, anyhow};
use crossbeam_channel::{Receiver, Sender, select};
use eframe::egui;
use ort::session::Session;

use crate::app::actions::{InferenceConfig, VisionAction};
use crate::core;

pub struct Frame {
    pub frame: core::Frame,
    pub overlay: Option<core::Frame>,
    pub intrinsics: core::Intrinsics,
}

pub struct InferenceEngine {
    model: Session,
    classes: Vec<String>,
    input_size: u32,
    prob_threshold: f32,
}

pub struct Runner {
    pub cmd_tx: Sender<VisionAction>,
    pub out_rx: Receiver<Result<Frame>>,
    handle: Option<JoinHandle<()>>,
}

pub struct Worker {
    camera: core::Camera,
    cmd_rx: Receiver<VisionAction>,
    out_tx: Sender<Result<Frame>>,

    egui_ctx: egui::Context,

    is_inference_enabled: bool,
    inference_engine: Option<InferenceEngine>,
}

impl TryFrom<InferenceConfig> for InferenceEngine {
    type Error = anyhow::Error;

    fn try_from(config: InferenceConfig) -> anyhow::Result<Self> {
        let model = Session::builder()?.commit_from_file(config.model_path)?;

        let contents = std::fs::read_to_string(config.classes_path)?;
        let classes = contents.lines().map(String::from).collect();

        Ok(Self {
            model,
            classes,
            input_size: config.input_size,
            prob_threshold: config.prob_threshold,
        })
    }
}

impl Runner {
    pub fn start(camera: core::Camera, egui_ctx: &egui::Context) -> Self {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (out_tx, out_rx) = crossbeam_channel::bounded(1);

        let mut worker = Worker {
            camera,
            cmd_rx,
            out_tx,
            egui_ctx: egui_ctx.clone(),
            is_inference_enabled: false,
            inference_engine: None,
        };

        let handle = std::thread::spawn(move || worker.run());

        Self {
            cmd_tx,
            out_rx,
            handle: Some(handle),
        }
    }
}

impl Worker {
    fn run(&mut self) {
        loop {
            select! {
                recv(self.cmd_rx) -> msg => {
                    match msg {
                        Ok(VisionAction::Stop) => {
                            log::info!("VisionAction::Stop");
                            break;
                        },
                        Ok(VisionAction::EnableInference{ config }) => {
                            log::info!("VisionAction::EnableInference");

                            if self.is_inference_enabled {
                                let e = anyhow!("Inference already enabled");
                                self.send_error_or_log(e);
                            } else {
                                match InferenceEngine::try_from(config) {
                                    Ok(engine) => self.inference_engine = Some(engine),
                                    Err(e) => {
                                        let e = anyhow!("Unable to load inference model: {e}");
                                        self.send_error_or_log(e);
                                    }
                                };
                                self.is_inference_enabled = true;
                            }
                        },
                        Ok(VisionAction::DisableInference) => {
                            log::info!("VisionAction::DisableInference");
                            self.is_inference_enabled = false;
                            self.inference_engine = None;
                        },
                        Err(e) => {
                            let e = anyhow!("Unable to receive VisionAction: {}", e);
                            self.send_error_or_log(e);
                        }
                    }
                }

                default(Duration::from_millis(0)) => {}
            }

            let (frame, intrinsics) = match self.camera.wait_for_frames() {
                Ok(f) => f,
                Err(e) => {
                    log::error!("Camera frame error: {}", e);
                    continue;
                }
            };

            if self.is_inference_enabled {
                let result = self.infere(frame.clone(), &intrinsics);
            }

            let vision = Frame {
                frame,
                overlay: None,
                intrinsics,
            };

            if self.out_tx.try_send(Ok(vision)).is_err() {
                continue;
            }

            self.egui_ctx.request_repaint();
        }
    }

    fn send_error_or_log(&self, e: anyhow::Error) {
        let _ = self
            .out_tx
            .try_send(Err(e))
            .inspect_err(|e| log::error!("{e}"));
    }

    fn infere(&mut self, frame: core::Frame, intrinsics: &core::Intrinsics) -> anyhow::Result<()> {
        let Some(engine) = self.inference_engine.as_mut() else {
            anyhow::bail!("Unable to infere without inference engine");
        };

        use ndarray::{Array, Axis, s};
        use ort::inputs;
        use ort::session::SessionOutputs;
        use ort::value::TensorRef;

        let letterbox = core::image::letterbox(
            frame.into_inner(),
            intrinsics.width as u32,
            intrinsics.height as u32,
            engine.input_size,
        )?;

        let input = core::image::input_array(letterbox);
        let input = TensorRef::from_array_view(&input)?;

        let outputs: SessionOutputs = engine.model.run(inputs!["images" => input])?;
        let output = outputs["output0"]
            .try_extract_array::<f32>()?
            .t()
            .into_owned();
        let output = output.slice(s![.., .., 0]);

        for row in output.axis_iter(Axis(0)) {
            let row: Vec<_> = row.iter().copied().collect();
            let (class_id, prob) = row
                .iter()
                .skip(4)
                .enumerate()
                .map(|(index, value)| (index, *value))
                .reduce(|accum, row| if row.1 > accum.1 { row } else { accum })
                .unwrap();
            if prob < engine.prob_threshold {
                continue;
            }

            let label = {
                if class_id < engine.classes.len() {
                    Some(engine.classes[class_id].clone())
                } else {
                    None
                }
            };
            label.inspect(|l| log::info!("Infered: {l}"));
        }

        Ok(())
    }
}
