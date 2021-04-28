use create_vox::node::*;
use std::fs::File;
use std::io::Read;

#[test]
fn node_add(){
    let mut node = Node{
        node_type: NodeType::Transform(Transform{
            layer: 0,
            rotation: 0,
            translation: (0, 0, 0)
        }),
        attributes: NodeAttributes{
            name: None,
            hidden: None
        },
        child: Vec::new()
    };

    node.add_child(Node::new(NodeType::Group));
    assert_eq!(node.child.len(), 1)
}

#[test]
fn make_tree(){
    let mut file = File::open("magicavoxel.vox").unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .expect("failed to read file contents");

    let node = create_vox::riff::nodes_from_chunks(&contents);

    println!("node is: {:?}", node);
    println!("number of nodes: {}", create_vox::riff::num_of_chunks(&contents, String::from("nTRN")) + create_vox::riff::num_of_chunks(&contents, String::from("nGRP")) + create_vox::riff::num_of_chunks(&contents, String::from("nSHP")));
    //println!("bench: {}", easybench::bench(|| {create_vox::riff::nodes_from_chunks(&contents);}))

    //recursion thing ¯\_(ツ)_/¯
    println!("shallow children: {}", node.child[0].child.len());
    println!("type: {:?}", node.node_type);
    println!("number of children is: {}", node.num_children());
}