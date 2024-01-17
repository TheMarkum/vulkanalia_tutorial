use anyhow::Result;
use vulkanalia::prelude::v1_0::*;

use crate::AppData;

pub unsafe fn create_framebuffers(device: &Device, data: &mut AppData) -> Result<()> {
    data.drawing_data.framebuffers = data
        .presentation_data
        .swapchain_image_views
        .iter()
        .map(|i| {
            let attachments = &[
                data.texture_data.color_image_view,
                data.texture_data.depth_image_view,
                *i,
            ];
            
            let create_info = vk::FramebufferCreateInfo::builder()
                .render_pass(data.pipeline_data.render_pass)
                .attachments(attachments)
                .width(data.presentation_data.swapchain_extent.width)
                .height(data.presentation_data.swapchain_extent.height)
                .layers(1);

            device.create_framebuffer(&create_info, None)
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(())
}
