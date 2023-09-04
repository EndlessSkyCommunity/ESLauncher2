use iced::Element;

use crate::advanced_frame;
use crate::instances_frame;
use crate::Message;

#[derive(Debug)]
pub enum AdvancedFrameOpen {
    Open(advanced_frame::AdvancedFrame),
    Closed
}

#[derive(Debug)]
pub struct InstanceFrameHolder {
    pub instances_frame: instances_frame::InstancesFrame,
    pub advanced_frame_open: AdvancedFrameOpen,
}

impl Default for InstanceFrameHolder {
    fn default() -> Self {
        InstanceFrameHolder {
            instances_frame: instances_frame::InstancesFrame::default(),
            advanced_frame_open: AdvancedFrameOpen::Closed
        }
    }
}

impl InstanceFrameHolder {
    pub fn view(&mut self) -> Element<Message> {
        match &mut self.advanced_frame_open {
            AdvancedFrameOpen::Open(advanced_frame) => {
                advanced_frame.view()
            },
            AdvancedFrameOpen::Closed => {
                self.instances_frame.view()
            }
        }
    }
}
