use create_vox::riff::{VoxString, find_chunk, nTRN};
use std::fs::File;
use std::io::Read;

#[test]
fn riff_string(){
    let content = &[2, 0, 0, 0, 104, 105];
    let my_string = VoxString::read(&content.to_vec(), &mut 0).content;

    assert_eq!(String::from("hi"), my_string);
}

#[test]
#[should_panic]
fn riff_string_fail(){
    let content = &[2, 0, 0, 0, 104, 105];
    let my_string = VoxString::read(&content.to_vec(), &mut 0).content;

    assert_eq!(String::from("HI"), my_string);
}

#[test]
fn chunk_read(){
    let mut file = File::open("magicavoxel.vox").unwrap();
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .expect("failed to read file contents");

    //start of first chunk
    let mut pos = create_vox::riff::find_chunk(&contents, String::from("nTRN"), 1).unwrap() as i32;
    let chunk = create_vox::riff::nTRN::read(&contents, &mut pos);

    println!("{:?}", chunk);
    println!("\n");
    println!("node id: {}", chunk.node_id);
    println!("node attributes: {:?}", chunk.node_attributes);
    println!("child node id: {}", chunk.child_node_id);
    println!("reserved id: {}", chunk.reserved_id);
    println!("layer id: {}", chunk.layer_id);
    println!("number of frames: {}", chunk.num_of_frames);
    println!("frame attributes: {:?}", chunk.frame_attributes);
}