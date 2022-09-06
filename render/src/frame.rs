use dom::node::NodePtr;
use gfx::Bitmap;
use layout::layout_box::LayoutBoxPtr;
use shared::primitive::{Point, Size};

use crate::pipeline::{Pipeline, PipelineRunOptions};

pub struct Frame {
    document: Option<NodePtr>,
    size: Size,
    bitmap: Option<Bitmap>,
}

impl Frame {
    pub fn new(init_size: Size) -> Self {
        Self {
            document: None,
            size: init_size,
            bitmap: None,
        }
    }

    pub fn size(&self) -> Size {
        self.size.clone()
    }

    pub async fn resize(&mut self, new_size: Size, pipeline: &mut Pipeline<'_>) {
        self.size = new_size.clone();
        self.render_frame(
            pipeline,
            PipelineRunOptions {
                skip_style_calculation: true,
                skip_layout_calculation: false,
            },
        )
        .await;
    }

    pub async fn handle_mouse_move(&self, coord: Point, pipeline: &mut Pipeline<'_>) {
        if let Some(root_node) = pipeline.content() {
            mark_mouse_over_boxes(root_node, &coord);
        }

        fn mark_mouse_over_boxes(root: LayoutBoxPtr, mouse_coord: &Point) {
            root.for_each_child(|child| {
                let child = LayoutBoxPtr(child);
                if child.border_box_absolute().is_contain_point(mouse_coord) {
                    child.set_mouse_over(true);
                } else {
                    child.set_mouse_over(false);
                }

                mark_mouse_over_boxes(child, mouse_coord);
            });
        }
    }

    pub async fn scroll(&mut self, delta_y: f32, pipeline: &mut Pipeline<'_>) {
        let mut need_redraw = false;

        // TODO: Handle scrolling for other overflow element within the current frame's document
        if let Some(root_node) = pipeline.content() {
            need_redraw = root_node.scroll(delta_y);
        }

        if !need_redraw {
            return;
        }

        self.render_frame(
            pipeline,
            PipelineRunOptions {
                skip_style_calculation: true,
                skip_layout_calculation: true,
            },
        )
        .await;
    }

    pub async fn set_document(&mut self, document: NodePtr, pipeline: &mut Pipeline<'_>) {
        self.document = Some(document.clone());
        self.render_frame(
            pipeline,
            PipelineRunOptions {
                skip_style_calculation: false,
                skip_layout_calculation: false,
            },
        )
        .await;
    }

    pub fn document(&self) -> Option<NodePtr> {
        self.document.clone()
    }

    pub fn bitmap(&self) -> Option<&Bitmap> {
        self.bitmap.as_ref()
    }

    async fn render_frame(&mut self, pipeline: &mut Pipeline<'_>, opts: PipelineRunOptions) {
        if let Some(document) = self.document() {
            let bitmap = pipeline.run(document, &self.size(), opts).await;
            self.bitmap = Some(bitmap);
        }
    }
}
