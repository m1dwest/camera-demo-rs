pub mod details;

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

pub struct Detection {
    pub rect: details::Rect,
    pub score: f32,
    pub label: Option<String>,
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
    _handle: Option<JoinHandle<()>>,
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
            _handle: Some(handle),
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

            let overlay = if self.is_inference_enabled {
                let detections = self.infere(frame.clone(), &intrinsics);
                match detections {
                    Ok(detections) => {
                        match details::generate_overlay(
                            intrinsics.width as u32,
                            intrinsics.height as u32,
                            detections,
                        ) {
                            Ok(overlay) => Some(overlay),
                            Err(e) => {
                                self.send_error_or_log(anyhow!(
                                    "Unable to generate overlay: {}",
                                    e
                                ));
                                None
                            }
                        }
                    }
                    Err(e) => {
                        self.send_error_or_log(anyhow!("Unable to infere: {}", e));
                        None
                    }
                }
            } else {
                None
            };

            let vision = Frame {
                frame,
                overlay,
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

    fn infere(
        &mut self,
        frame: core::Frame,
        intrinsics: &core::Intrinsics,
    ) -> anyhow::Result<Vec<Detection>> {
        let Some(engine) = self.inference_engine.as_mut() else {
            anyhow::bail!("Unable to infere without inference engine");
        };

        use ndarray::{Array, Axis, s};
        use ort::inputs;
        use ort::session::SessionOutputs;
        use ort::value::TensorRef;

        let letterbox = details::Letterbox::from_vec(
            frame.into_inner(),
            intrinsics.width as u32,
            intrinsics.height as u32,
            engine.input_size,
        )?;

        let input = details::input_array(&letterbox.rgb);
        let input = TensorRef::from_array_view(&input)?;

        let outputs: SessionOutputs = engine.model.run(inputs!["images" => input])?;
        let output = outputs["output0"].try_extract_array::<f32>()?; // [1, 84, 8400]
        let output = output.index_axis(Axis(0), 0); // [84, 8400]
        let output = output.t(); // [8400, 84]

        let detections: Vec<Detection> = output
            .axis_iter(Axis(0))
            .filter_map(|pred| {
                let scores = pred.slice(s![4..]);
                let (class_id, score) = argmax(scores.iter());

                if score < engine.prob_threshold {
                    return None;
                }

                let label = {
                    if class_id < engine.classes.len() {
                        Some(engine.classes[class_id].clone())
                    } else {
                        None
                    }
                };

                let rect = details::Rect {
                    x: pred[0],
                    y: pred[1],
                    w: pred[2],
                    h: pred[3],
                };
                let rect = letterbox.yolo_rect_to_src(&rect);

                Some(Detection { rect, score, label })
            })
            .collect();

        Ok(detections)
    }
}

fn argmax<'a, I>(it: I) -> (usize, f32)
where
    I: Iterator<Item = &'a f32>,
{
    it.enumerate()
        .fold((0, f32::NEG_INFINITY), |best, (i, &v)| {
            if v > best.1 { (i, v) } else { best }
        })
}
