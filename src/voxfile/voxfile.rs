use crate::model::Model;
use crate::Color;
use crate::node::{Node, NodeType, Transform, NodeAttributes};
use crate::layer::Layer;

pub struct VoxFile{
    pub models: Vec<Model>,
    pub palette: [Color; 256],
    pub root_node: Node,
    pub layers: Vec<Layer>
}

impl VoxFile{
    //size in bytes when written
    pub(in crate::voxfile) fn get_size(&self) -> i32{
        let mut size = 1024;
        for model in self.models.iter(){
            size += model.get_size();
        }
        for layer in self.layers.iter(){
            size += layer.get_size()
        }
        size += self.root_node.get_all_size();
        size
    }

    pub(in crate::voxfile) fn make_nodes(&mut self) {
        let mut root_node = Node::new(NodeType::Transform(Transform::default()), NodeAttributes::new());
        let mut group_node = Node::new(NodeType::Group, NodeAttributes::new());

        for model in self.models.iter() {
            group_node.add_child(model.to_node());
        }
        root_node.add_child(group_node);
        self.root_node = root_node
    }

    //takes data from nodes and applies it to models
    pub fn get_node_data(&mut self) {
        //need to change
        self.root_node.clone().get_child_data_to_models(self)
    }

    //(id, pos, layer)
    pub fn check_transform(transform_node: &Node) -> Option<(i32, Option<(i32, i32, i32)>, Option<i32>)>{
        let id: i32;
        let pos: Option<(i32, i32, i32)>;
        let layer: Option<i32>;

        match transform_node.node_type {
            NodeType::Transform(ref trans) => {
                pos = trans.translation;
                layer = Some(trans.layer)
            }
            _ => {return None}
        }

        if transform_node.has_child_shape() {
            match transform_node.children[0].node_type {
                NodeType::Shape(shape_id) => {
                    id = shape_id;
                }
                _ => {return None}
            }
        } else {
            return None
        }

        Some((id, pos, layer))
    }

    //finds model by id and edits it with given data
    pub fn change_model_data(&mut self, id: i32, pos: Option<(i32, i32, i32)>, layer: Option<i32>){
        for model in self.models.iter_mut() {
            if model.id == id {
                model.position = pos;
                model.layer = layer;
            }
        }
    }
}