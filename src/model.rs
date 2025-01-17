use crate::convert::*;
use crate::node::{Node, NodeAttributes, NodeType, Transform};
use crate::riff::write_chunk;
use crate::writing::*;
use crate::*;
use std::fs::File;
use std::io::BufWriter;

/// Holds voxel data
#[derive(Clone)]
pub struct Model {
    pub size: (u16, u16, u16),
    pub voxels: Vec<Voxel>,
    pub position: Option<(i32, i32, i32)>,
    pub rotation: Option<u8>,
    pub layer: Option<i32>,
    pub name: Option<String>,
    pub(crate) id: i32,
}

#[allow(unused_variables)]
#[allow(dead_code)]
impl Model {
    /// Creates a new model with the size given
    ///
    /// # Example
    /// ```
    /// use create_vox::{VoxFile, Model};
    ///
    /// let mut vox = VoxFile::new(5,5,5);
    ///
    /// // adds a new model to the voxfile with a size 10 by 10 by 10
    /// vox.models.push(Model::new(10, 10, 10));
    /// ```
    pub fn new(x: u16, y: u16, z: u16) -> Model {
        Model {
            size: (x, y, z),
            voxels: Vec::new(),
            position: None,
            rotation: None,
            layer: None,
            name: None,
            id: 0,
        }
    }

    pub(crate) fn write(&self, writer: &mut BufWriter<File>) {
        let size_slice: &[u8] = &[
            u16_to_array(self.size.0)[0],
            u16_to_array(self.size.0)[1],
            0,
            0,
            u16_to_array(self.size.1)[0],
            u16_to_array(self.size.1)[1],
            0,
            0,
            u16_to_array(self.size.2)[0],
            u16_to_array(self.size.2)[1],
            0,
            0,
        ];
        write_chunk("SIZE", 12, 0, writer);
        //writes the slice for size
        write_slice(writer, size_slice);

        write_chunk("XYZI", ((self.voxels.len() as u32) * 4) + 4, 0, writer);
        //number voxels in the voxobject
        write_slice(writer, &u32_to_array(self.voxels.len() as u32));
        //writes all of the voxels
        self.write_voxels(writer);
    }

    fn write_voxels(&self, buf_writer: &mut BufWriter<File>) {
        let mut voxel_slice: Box<Vec<u8>> = Box::new(vec![]);
        for i in 0..self.voxels.len() {
            voxel_slice.push(self.voxels[i].position.0);
            voxel_slice.push(self.voxels[i].position.1);
            voxel_slice.push(self.voxels[i].position.2);
            voxel_slice.push(self.voxels[i].color_index);
        }
        buf_writer.write(voxel_slice.as_slice()).unwrap();
    }

    //start at size chunk
    pub(crate) fn read(input: &Vec<u8>, cursor: &mut i32, id: i32) -> Model {
        use crate::riff::i32_from_vec;
        *cursor += 12;
        let size_x = i32_from_vec(input, cursor) as u16;
        *cursor += 4;
        let size_y = i32_from_vec(input, cursor) as u16;
        *cursor += 4;
        let size_z = i32_from_vec(input, cursor) as u16;
        *cursor += 16;

        let num_of_voxels = i32_from_vec(input, cursor);
        *cursor += 4;
        let mut voxels = Vec::new();
        for i in 0..num_of_voxels {
            let x = input[(*cursor + 4 * i) as usize];
            let y = input[(*cursor + 1 + 4 * i) as usize];
            let z = input[(*cursor + 2 + 4 * i) as usize];
            let index = input[(*cursor + 3 + 4 * i) as usize];
            voxels.push(Voxel::new(x, y, z, index))
        }

        Model {
            size: (size_x, size_y, size_z),
            voxels,
            position: None,
            rotation: None,
            layer: None,
            name: None,
            id,
        }
    }

    pub(crate) fn to_node(&self) -> Node {
        let mut attributes = NodeAttributes::new();
        attributes.name = self.name.clone();
        let mut transform_node = Node::new(NodeType::Transform(self.transform_data()), attributes);
        let shape_node = Node::new(NodeType::Shape(self.id), NodeAttributes::new());
        transform_node.add_child(shape_node);

        transform_node
    }

    //puts data into Transform struct
    pub(crate) fn transform_data(&self) -> Transform {
        Transform {
            layer: self.layer.unwrap_or_else(|| 0),
            rotation: match self.rotation {
                None => None,
                Some(rot) => Some(rot as i32),
            },
            translation: self.position,
        }
    }

    //size in bytes when written
    pub(crate) fn get_size(&self) -> i32 {
        self.voxels.len() as i32 * 4 + 4
    }

    //start of functions for users.

    /// Adds a voxel to the model. It will return an error if the voxel does not fit inside the model.
    ///
    /// # Example
    /// ```
    /// use create_vox::{VoxFile, Voxel};
    ///
    /// let mut vox = VoxFile::new(10,10,10);
    /// let voxel = Voxel::new(4, 2, 2, 10);
    /// vox.models[0].add_voxel(voxel);
    /// ```
    pub fn add_voxel(&mut self, new_voxel: Voxel) -> Result<(), &str> {
        if (new_voxel.position.0 + 1) as u16 > self.size.0
            || (new_voxel.position.1 + 1) as u16 > self.size.1
            || (new_voxel.position.2 + 1) as u16 > self.size.2
        {
            return Err("Voxel position greater than Voxobject size");
        }
        self.voxels.push(new_voxel);
        Ok(())
    }

    /// Makes the size of the model as small as possible
    ///
    /// # Example
    /// ```
    /// use create_vox::VoxFile;
    ///
    /// let mut vox = VoxFile::new(10,10,10);
    /// vox.models[0].add_cube(2, 2, 2, 10, 10, 10, 1);
    /// vox.models[0].clear_voxels();
    /// assert_eq!(0, vox.models[0].num_of_voxels());
    /// ```
    pub fn clear_voxels(&mut self) {
        self.voxels.clear();
    }

    /// Sets the size of the model. Size must be less than or equal to 256 on all axis.
    ///
    /// # Example
    /// ```
    /// use create_vox::VoxFile;
    ///
    /// let mut vox = VoxFile::new(20,20,20);
    /// vox.models[0].set_size(12,6,24);
    /// assert_eq!(vox.models[0].size, (12, 6, 24));
    /// ```
    pub fn set_size(&mut self, x: u16, y: u16, z: u16) {
        if x > 256 || y > 256 || z > 256 {
            panic!("size can not be greater than 256");
        }
        self.size = (x, y, z);
    }

    /// Makes the size of the model as small as possible
    ///
    /// # Example
    /// ```
    /// use create_vox::VoxFile;
    ///
    /// let mut vox = VoxFile::new(10,10,10);
    /// vox.models[0].add_cube(2, 2, 2, 6, 7, 6, 1);
    /// vox.models[0].auto_size();
    /// ```
    pub fn auto_size(&mut self) {
        let mut new_size = (1, 1, 1);
        let mut smallest_pos: (u8, u8, u8) = (255, 255, 255);

        //get smallest position of the voxels
        for voxel in self.voxels.iter() {
            if voxel.position.0 < smallest_pos.0 {
                smallest_pos.0 = voxel.position.0
            }
            if voxel.position.1 < smallest_pos.1 {
                smallest_pos.1 = voxel.position.1
            }
            if voxel.position.2 < smallest_pos.2 {
                smallest_pos.2 = voxel.position.2
            }
        }
        //move voxels
        for voxel in self.voxels.iter_mut() {
            voxel.position = (
                voxel.position.0 - smallest_pos.0,
                voxel.position.1 - smallest_pos.1,
                voxel.position.2 - smallest_pos.2,
            )
        }

        for voxel in self.voxels.iter() {
            if (voxel.position.0 as u16) > new_size.0 - 1 {
                new_size.0 = (voxel.position.0 + 1) as u16
            }
            if (voxel.position.1 as u16) > new_size.1 - 1 {
                new_size.1 = (voxel.position.1 + 1) as u16
            }
            if (voxel.position.2 as u16) > new_size.2 - 1 {
                new_size.2 = (voxel.position.2 + 1) as u16
            }
        }

        self.size = new_size
    }

    /// Fills in the area between 2 points with voxels
    ///
    /// # Example
    /// ```
    /// use create_vox::VoxFile;
    ///
    /// let mut vox = VoxFile::new(10,10,10);
    /// vox.models[0].add_cube(0, 0, 0, 5, 5, 5, 1);
    /// ```
    pub fn add_cube(
        &mut self,
        startx: u8,
        starty: u8,
        startz: u8,
        endx: u8,
        endy: u8,
        endz: u8,
        colorindex: u8,
    ) -> Result<(), &str> {
        if endx as u16 > self.size.0 || endx as u16 > self.size.1 || endx as u16 > self.size.2 {
            return Err("Cube too large");
        }
        for currentx in startx..endx {
            for currenty in starty..endy {
                for currentz in startz..endz {
                    self.add_voxel(Voxel::new(currentx, currenty, currentz, colorindex))
                        .unwrap();
                }
            }
        }

        Ok(())
    }

    /// Checks if there is a voxel at the position
    ///
    /// # Example
    /// ```
    /// use create_vox::VoxFile;
    ///
    /// let mut vox = VoxFile::new(10,10,10);
    /// vox.models[0].add_voxel_at_pos(3,4,3,1);
    /// assert_eq!(true, vox.models[0].is_voxel_at_pos(3, 4, 3));
    /// ```
    pub fn is_voxel_at_pos(&self, x: u8, y: u8, z: u8) -> bool {
        for voxel in self.voxels.iter() {
            if voxel.position.0 == x && voxel.position.1 == y && voxel.position.2 == z {
                return true;
            }
        }
        false
    }

    //needs testing
    fn check_voxels_pos(&mut self) {
        let size = self.size;
        self.voxels.retain(|voxel| {
            (voxel.position.0 as u16) < size.0
                && (voxel.position.1 as u16) < size.1
                && (voxel.position.2 as u16) < size.2
        });
    }

    /// Adds a voxel at certain position
    ///
    /// # Example
    /// ```
    /// use create_vox::VoxFile;
    ///
    /// let mut vox = VoxFile::new(50,10,30);
    /// vox.models[0].add_voxel_at_pos(1,1,1,6).unwrap();
    /// vox.models[0].add_voxel_at_pos(1,1,2,5).unwrap();
    /// vox.models[0].add_voxel_at_pos(1,1,3,6).unwrap();
    /// vox.models[0].add_voxel_at_pos(1,1,4,7).unwrap();
    ///
    /// vox.models[0].retain_voxels(|voxel| voxel.color_index == 6);
    ///
    /// assert_eq!(2, vox.models[0].num_of_voxels());
    /// ```
    pub fn add_voxel_at_pos(&mut self, x: u8, y: u8, z: u8, voxel_index: u8) -> Result<(), &str> {
        if (x + 1) as u16 > self.size.0
            || (y + 1) as u16 > self.size.1
            || (z + 1) as u16 > self.size.2
        {
            return Err("Position greater than Voxobject size");
        }
        self.voxels.push(Voxel::new(x, y, z, voxel_index));
        Ok(())
    }

    /// Returns the number of voxels in the model
    ///
    /// # Example
    /// ```
    /// use create_vox::VoxFile;
    ///
    /// let mut vox = VoxFile::new(10,10,10);
    /// vox.models[0].add_voxel_at_pos(8,1,1,1).unwrap();
    /// vox.models[0].add_voxel_at_pos(3,3,2,1).unwrap();
    /// vox.models[0].add_voxel_at_pos(6,1,3,2).unwrap();
    /// vox.models[0].add_voxel_at_pos(1,1,4,2).unwrap();
    ///
    /// assert_eq!(4, vox.models[0].num_of_voxels());
    /// ```
    pub fn num_of_voxels(&self) -> i32 {
        self.voxels.len() as i32
    }

    /// Keeps all of the voxels in the Voxobject that return true with the closure given
    ///
    /// # Example
    /// ```
    /// use create_vox::VoxFile;
    ///
    /// let mut vox = VoxFile::new(10,10,10);
    /// vox.models[0].add_voxel_at_pos(1,1,1,6).unwrap();
    /// vox.models[0].add_voxel_at_pos(1,1,2,5).unwrap();
    /// vox.models[0].add_voxel_at_pos(1,1,3,6).unwrap();
    /// vox.models[0].add_voxel_at_pos(1,1,4,7).unwrap();
    ///
    /// vox.models[0].retain_voxels(|voxel| voxel.color_index == 6);
    ///
    /// assert_eq!(2, vox.models[0].num_of_voxels());
    /// ```
    pub fn retain_voxels<T>(&mut self, closure: T)
    where
        T: FnMut(&Voxel) -> bool,
    {
        self.voxels.retain(closure);
    }

    /// Changes all the voxels in the Voxobject with the closure
    ///
    /// # Example
    /// ```
    /// use create_vox::VoxFile;
    ///
    /// let mut vox = VoxFile::new(10, 10, 10);
    /// vox.models[0].add_voxel_at_pos(1,1,1,6).unwrap();
    /// vox.models[0].add_voxel_at_pos(1,1,2,5).unwrap();
    /// vox.models[0].add_voxel_at_pos(1,1,3,6).unwrap();
    /// vox.models[0].add_voxel_at_pos(1,1,4,7).unwrap();
    ///
    /// //make all voxels have index 3 on the palette as their color
    /// vox.models[0].change_voxels(|voxel| voxel.color_index = 3);
    /// ```
    pub fn change_voxels<T>(&mut self, mut closure: T)
    where
        T: FnMut(&mut Voxel),
    {
        let voxel_iter = self.voxels.iter_mut();

        for voxel in voxel_iter {
            closure(voxel);
        }
    }

    pub fn get_id(&self)-> i32{
        self.id
    }
}
