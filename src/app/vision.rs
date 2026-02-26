use std::{thread, time::Duration};

use anyhow::{Result, anyhow};
use crossbeam_channel::{Receiver, Sender, select};
use eframe::egui;
use thread::JoinHandle;

use crate::app::actions::VisionAction;
use crate::core;

pub struct Frame {
    pub frame: core::Frame,
    pub overlay: Option<core::Frame>,
    pub intrinsics: core::Intrinsics,
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
                            let error = Err(anyhow!("Inference already enabled"));
                            self.out_tx.try_send(error);
                        }
                            self.is_inference_enabled = true;
                        },
                        Ok(VisionAction::DisableInference) => {
                            log::info!("VisionAction::DisableInference");
                            self.is_inference_enabled = false;
                        },
                        Err(e) => {
                            let error = Err(anyhow!("Unable to receive VisionAction: {}", e));
                            self.out_tx.try_send(error);
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
}

const YOLOV8_CLASS_LABELS: [&str; 80] = [
    "person",
    "bicycle",
    "car",
    "motorcycle",
    "airplane",
    "bus",
    "train",
    "truck",
    "boat",
    "traffic light",
    "fire hydrant",
    "stop sign",
    "parking meter",
    "bench",
    "bird",
    "cat",
    "dog",
    "horse",
    "sheep",
    "cow",
    "elephant",
    "bear",
    "zebra",
    "giraffe",
    "backpack",
    "umbrella",
    "handbag",
    "tie",
    "suitcase",
    "frisbee",
    "skis",
    "snowboard",
    "sports ball",
    "kite",
    "baseball bat",
    "baseball glove",
    "skateboard",
    "surfboard",
    "tennis racket",
    "bottle",
    "wine glass",
    "cup",
    "fork",
    "knife",
    "spoon",
    "bowl",
    "banana",
    "apple",
    "sandwich",
    "orange",
    "broccoli",
    "carrot",
    "hot dog",
    "pizza",
    "donut",
    "cake",
    "chair",
    "couch",
    "potted plant",
    "bed",
    "dining table",
    "toilet",
    "tv",
    "laptop",
    "mouse",
    "remote",
    "keyboard",
    "cell phone",
    "microwave",
    "oven",
    "toaster",
    "sink",
    "refrigerator",
    "book",
    "clock",
    "vase",
    "scissors",
    "teddy bear",
    "hair drier",
    "toothbrush",
];

// fn infere(frame: &Frame, intrinsics: &Intrinsics) -> anyhow::Result<()> {
//     use ndarray::{Array, Axis, s};
//     use ort::inputs;
//     use ort::session::{Session, SessionOutputs, builder::GraphOptimizationLevel};
//     use ort::value::TensorRef;
//
//     // let model = Session::builder()
//     //     .context("Unable to create ORT Session builder")?
//     //     .with_optimization_level(GraphOptimizationLevel::Level3)?
//     //     .with_intra_threads(4)?
//     //     .commit_from_file("yolov12n.onnx")?;
//
//     use crate::core::image::{letterbox, to_nchw_f32};
//
//     let target_size: u32 = 640;
//     let lb = letterbox(
//         frame.into_inner(),
//         intrinsics.width as u32,
//         intrinsics.height as u32,
//         target_size,
//     )?;
//
//     let mut input: ndarray::Array4<f32> =
//         Array::zeros((1, 3, target_size as usize, target_size as usize));
//     for (i, px) in lb.pixels().enumerate() {
//         let y: u32 = i as u32 / target_size as u32;
//         let x = i as u32 - (y * target_size as u32);
//         let r = px[0] as f32 / 255.0;
//         let g = px[1] as f32 / 255.0;
//         let b = px[2] as f32 / 255.0;
//         input[[0, 0, y as usize, x as usize]] = r;
//         input[[0, 1, y as usize, x as usize]] = g;
//         input[[0, 2, y as usize, x as usize]] = b;
//     }
//
//     // let nchw = to_nchw_f32(lb);
//
//     if self.model.is_none() {
//         self.model = Some(Session::builder()?.commit_from_file("yolov12n.onnx")?);
//     }
//
//     let model = self.model.as_mut().unwrap();
//     let outputs: SessionOutputs =
//         model.run(inputs!["images" => TensorRef::from_array_view(&input)?])?;
//     let output = outputs["output0"]
//         .try_extract_array::<f32>()?
//         .t()
//         .into_owned();
//     let output = output.slice(s![.., .., 0]);
//     for row in output.axis_iter(Axis(0)) {
//         let row: Vec<_> = row.iter().copied().collect();
//         let (class_id, prob) = row
//             .iter()
//             .skip(4)
//             .enumerate()
//             .map(|(index, value)| (index, *value))
//             .reduce(|accum, row| if row.1 > accum.1 { row } else { accum })
//             .unwrap();
//         if prob < 0.5 {
//             continue;
//         }
//         let label = App::YOLOV8_CLASS_LABELS[class_id];
//     }
//
//     Ok(())
// }
