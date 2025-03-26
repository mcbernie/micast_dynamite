use crate::vdom::DiffOp;

pub trait Renderer {
    fn render(&mut self, node: &DiffOp);
}

/* 
impl VDom {
    pub fn render_snapshot(&self) -> RenderNode {
        Self::collect_render_data(&self.root)
    }

    fn collect_render_data(node: &Rc<RefCell<VNode>>) -> RenderNode {
        match &*node.borrow() {
            VNode::Element { tag, styles, children, .. } => {
                let child_render_nodes = children
                    .iter()
                    .map(Self::collect_render_data)
                    .collect();

                RenderNode::Element {
                    tag: tag.clone(),
                    styles: styles.clone(),
                    children: child_render_nodes,
                }
            }
            VNode::Text { rendered, .. } => RenderNode::Text(rendered.clone()),
        }
    }
}
*/