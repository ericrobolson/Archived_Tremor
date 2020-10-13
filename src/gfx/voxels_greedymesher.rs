use cgmath::Vector3;

// Simple file for reference on greedy meshing. Not exactly sane.

#[derive(Debug, Clone, PartialEq)]
pub struct Mesh {
    pub indices: Vec<i32>,
    /// The number of values each vertex is composed of. Can be 1, 2, 3, or 4. TODO: make this some sort of static, fixed thing.
    pub vertex_size: usize,
    pub vertices: Vec<f32>,
    pub colors: Vec<f32>,
    pub color_vertex_size: usize,
    pub normals: Vec<f32>,
    pub normal_vertex_size: usize,
    pub generated_at_frame: usize,
}

impl Mesh {
    pub fn new(
        vertex_size: usize,
        vertices: Vec<f32>,
        indices: Vec<i32>,
        color_vertex_size: usize,
        colors: Vec<f32>,
        normal_vertex_size: usize,
        normals: Vec<f32>,
        generated_at_frame: usize,
    ) -> Self {
        return Self {
            vertex_size: vertex_size,
            vertices: vertices,
            indices: indices,
            color_vertex_size: color_vertex_size,
            colors: colors,
            normal_vertex_size: normal_vertex_size,
            normals: normals,
            generated_at_frame: generated_at_frame,
        };
    }

    pub fn is_empty(&self) -> bool {
        return self.vertices.is_empty() && self.indices.is_empty();
    }

    pub fn merge(meshes: &Vec<Mesh>, frame: usize) -> Mesh {
        //TODO: paralellize
        let mut mesh = Mesh::new(3, vec![], vec![], 3, vec![], 3, vec![], 0);
        if meshes.is_empty() == false {
            let mut is_first = true;
            let mut offset = 0;

            for m in meshes.iter() {
                if is_first {
                    mesh.vertex_size = m.vertex_size;
                    mesh.color_vertex_size = m.color_vertex_size;
                    mesh.normal_vertex_size = m.normal_vertex_size;
                    is_first = false;
                }

                if mesh.vertex_size != m.vertex_size {
                    panic!("Unable to merge meshes! Mesh vertex sizes differ.");
                }

                if mesh.normal_vertex_size != m.normal_vertex_size {
                    panic!("Unable to merge meshes! Mesh normal vertex sizes differ.");
                }

                if mesh.color_vertex_size != m.color_vertex_size {
                    panic!("Unable to merge meshes! Mesh color vertex sizes differ.");
                }

                mesh.vertices.append(&mut m.vertices.clone());
                mesh.colors.append(&mut m.colors.clone());
                mesh.normals.append(&mut m.normals.clone());

                // do tricky shit with indices
                let mut mapped_indices = m.indices.iter().map(|i| i + offset).collect();

                mesh.indices.append(&mut mapped_indices);

                if m.vertex_size != 0 {
                    offset += m.vertices.len() as i32 / m.vertex_size as i32;
                }

                if m.generated_at_frame > mesh.generated_at_frame {
                    mesh.generated_at_frame = m.generated_at_frame;
                }
            }
        }

        if mesh.is_empty() {
            mesh.generated_at_frame = frame;
        }

        return mesh;
    }
}

pub fn calculate_greedy_mesh(
    voxels: &[[[CbVoxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    chunk_x_offset: usize,
    chunk_y_offset: usize,
    chunk_z_offset: usize,
    frame: usize,
    chunk_size: usize,
) -> Mesh {
    const SOUTH: usize = 0;
    const NORTH: usize = 1;
    const EAST: usize = 2;
    const WEST: usize = 3;
    const TOP: usize = 4;
    const BOTTOM: usize = 5;

    let chunk_width: usize = chunk_size;
    let chunk_width_i: i32 = chunk_width as i32;
    let chunk_height: usize = chunk_size;
    let chunk_height_i: i32 = chunk_height as i32;

    let mut meshes = vec![];

    // Referenced https://github.com/roboleary/GreedyMesh/blob/master/src/mygame/Main.java

    // Create the working variables
    let_mut_for![(i, j, k, l, w, h, u, v, n, side), usize, 0];
    let_mut_for![(x, q, du, dv), Vector3<i32>, Vector3::new(0, 0, 0)];

    // Create a mask of matching voxel faces as we go through the chunk in 6 directions, once for each face
    let mut mask = Vec::with_capacity(chunk_width * chunk_height);
    for _ in 0..chunk_width * chunk_height {
        mask.push(None);
    }

    // Working variables to hold two faces during comparison
    let_mut_for![(voxel_face, voxel_face1), Option<VoxelFace>, None];

    let mut backface = false;

    // First loop run it with the backface, second loop run it without. This allows us to track the directions the indices should run during the creation of the quad.
    // Outer loop will run twice, inner loop 3 times. Once for each voxel face.
    for _ in 0..2 {
        backface = !backface;

        // Sweep over the 3 dimensions to mesh it.

        for d in 0..3 {
            // Set variables
            {
                u = (d + 1) % 3;
                v = (d + 2) % 3;

                x[0] = 0;
                x[1] = 0;
                x[2] = 0;

                q[0] = 0;
                q[1] = 0;
                q[2] = 0;
                q[d] = 1;
            }

            // Keep track of the side that is being meshed.
            {
                if d == 0 {
                    if backface {
                        side = WEST;
                    } else {
                        side = EAST;
                    }
                } else if d == 1 {
                    if backface {
                        side = BOTTOM;
                    } else {
                        side = TOP;
                    }
                } else if d == 2 {
                    if backface {
                        side = SOUTH;
                    } else {
                        side = NORTH;
                    }
                }
            }

            // Move through the dimension from front to back
            x[d] = -1;
            while x[d] < chunk_width_i {
                // Compute the mask
                {
                    n = 0;

                    x[v] = 0;
                    while x[v] < chunk_height_i {
                        x[u] = 0;
                        while x[u] < chunk_width_i {
                            // Retrieve the two voxel faces to compare.
                            if x[d] >= 0 {
                                voxel_face = Some(get_voxel_face(
                                    voxels,
                                    x[0] as usize,
                                    x[1] as usize,
                                    x[2] as usize,
                                    side,
                                    chunk_size - 1,
                                    chunk_size,
                                ));
                            } else {
                                voxel_face = None;
                            }

                            if x[d] < chunk_width_i - 1 {
                                voxel_face1 = Some(get_voxel_face(
                                    voxels,
                                    (x[0] + q[0]) as usize,
                                    (x[1] + q[1]) as usize,
                                    (x[2] + q[2]) as usize,
                                    side,
                                    chunk_size - 1,
                                    chunk_size,
                                ));
                            } else {
                                voxel_face1 = None;
                            }
                            // Compare the faces based on number of attributes. Choose the face to add to the mask depending on backface or not.
                            if voxel_face.is_some()
                                && voxel_face1.is_some()
                                && voxel_face.unwrap().equals(&voxel_face1.unwrap())
                            {
                                mask[n] = None;
                            } else if backface {
                                mask[n] = voxel_face1;
                            } else if !backface {
                                mask[n] = voxel_face;
                            }

                            n += 1;
                            x[u] += 1;
                        }

                        x[v] += 1;
                    }
                }

                x[d] += 1;

                // Now generate the mesh for the mask
                n = 0;
                j = 0;
                while j < chunk_height {
                    i = 0;
                    while i < chunk_width {
                        if mask[n].is_some() {
                            // Compute the width
                            w = 1;
                            while i + w < chunk_width
                                && mask[n + w].is_some()
                                && mask[n + w].unwrap().equals(&mask[n].unwrap())
                            {
                                w += 1;
                            }

                            // Compute the height
                            let mut done = false;

                            h = 1;
                            while j + h < chunk_height {
                                k = 0;
                                while k < w {
                                    if mask[n + k + h * chunk_width].is_none()
                                        || !mask[n + k + h * chunk_width]
                                            .unwrap()
                                            .equals(&mask[n].unwrap())
                                    {
                                        done = true;
                                        break;
                                    }
                                    k += 1;
                                }

                                if done {
                                    break;
                                }
                                h += 1;
                            }

                            // Do not mesh transparent/culled faces
                            if !mask[n].unwrap().transparent {
                                // Add quad
                                x[u] = i as i32;
                                x[v] = j as i32;

                                du[0] = 0;
                                du[1] = 0;
                                du[2] = 0;
                                du[u] = w as i32;

                                dv[0] = 0;
                                dv[1] = 0;
                                dv[2] = 0;
                                dv[v] = h as i32;

                                let x0 = x[0] as f32;
                                let x1 = x[1] as f32;
                                let x2 = x[2] as f32;

                                let du0 = du[0] as f32;
                                let du1 = du[1] as f32;
                                let du2 = du[2] as f32;

                                let dv0 = dv[0] as f32;
                                let dv1 = dv[1] as f32;
                                let dv2 = dv[2] as f32;

                                // Call the quad() to render the merged quad in the scene. mask[n] will contain the attributes to pass to shaders
                                let quad = get_quad(
                                    chunk_x_offset,
                                    chunk_y_offset,
                                    chunk_z_offset,
                                    Vector3::new(x0, x1, x2),
                                    Vector3::new(x0 + du0, x1 + du1, x2 + du2),
                                    Vector3::new(x0 + du0 + dv0, x1 + du1 + dv1, x2 + du2 + dv2),
                                    Vector3::new(x0 + dv0, x1 + dv1, x2 + dv2),
                                    w,
                                    h,
                                    mask[n].unwrap(),
                                    backface,
                                    frame,
                                );

                                meshes.push(quad);
                            }

                            // Zero out the mask
                            l = 0;
                            while l < h {
                                k = 0;
                                while k < w {
                                    mask[n + k + l * chunk_width] = None;

                                    k += 1;
                                }

                                l += 1;
                            }

                            // Increment the counters + continue
                            i += w;
                            n += w;
                        } else {
                            i += 1;
                            n += 1;
                        }
                    }
                    j += 1;
                }
            }
        }
    }

    return Mesh::merge(&meshes, frame);
}

fn get_voxel_face(
    voxels: &[[[CbVoxel; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
    x: usize,
    y: usize,
    z: usize,
    side: usize,
    chunk_size: usize,
    max_index: usize,
) -> VoxelFace {
    // NOTE: Add the following here:
    // ** Set per face / per vertex values as well as voxel values here.

    let (active, visible, vf_type, _) = voxels[x][y][z];

    let mut transparent = !active;

    /*
    NOTE: THIS PART IS BUGGY AND DOESN"T WORK
    */
    
    let check_for_obscured = false;
    if check_for_obscured {
        /*
        // Check neighbors to see if obscured and cull if so
        if (x != 0 && x != max_index) && (y != 0 && y != max_index) && (z != 0 && z != max_index) {
            // above layer
            let i = x + y * chunk_size + (z - 1) * chunk_size_squared;
            let (obscured_above, _) = voxels[i]; //  let obscured_above = voxels[x][y][z - 1].active;
            // same layer
            let i1 = x + (y + 1) * chunk_size + z * chunk_size_squared;
            let i2 = x + (y - 1) * chunk_size + z * chunk_size_squared;
            let i3 = (x + 1) + y * chunk_size + z * chunk_size_squared;
            let i4 = (x - 1) + y * chunk_size + z * chunk_size_squared;
            let obscured_same = voxels[i1].0 // voxels[x][y + 1][z].active
            && voxels[i2].0 // voxels[x][y - 1][z].active
            && voxels[i3].0 // voxels[x + 1][y][z].active
            && voxels[i4].0; // voxels[x - 1][y][z].active;
            // below layer
            let i = x + y * chunk_size + (z + 1) * chunk_size_squared;
            let (obscured_below, _) = voxels[i];
            if !transparent && obscured_above && obscured_same && obscured_below {
                // transparent = true;
            }
        }*/
    }
    return VoxelFace {
        transparent: transparent,
        vf_type: vf_type,
        side: side,
    };
}

type V3 = nalgebra::Matrix<
    f32,
    nalgebra::U3,
    nalgebra::U1,
    nalgebra::ArrayStorage<f32, nalgebra::U3, nalgebra::U1>,
>;

fn get_quad(
    chunk_x_offset: usize,
    chunk_y_offset: usize,
    chunk_z_offset: usize,
    bottom_left: V3,
    top_left: V3,
    top_right: V3,
    bottom_right: V3,
    width: usize,
    height: usize,
    voxel: VoxelFace,
    backface: bool,
    generated_at_frame: usize,
) -> Mesh {
    const VALUES_IN_VERTEX: usize = 3;
    let vertices;
    let indices;
    {
        let x_offset = (chunk_x_offset * CHUNK_SIZE) as f32 * VOXEL_SIZE;
        let y_offset = (chunk_y_offset * CHUNK_SIZE) as f32 * VOXEL_SIZE;
        let z_offset = (chunk_z_offset * CHUNK_SIZE) as f32 * VOXEL_SIZE;

        vertices = vec![
            // ----
            bottom_left.x + x_offset,
            bottom_left.y + y_offset,
            bottom_left.z + z_offset,
            // ----
            bottom_right.x + x_offset,
            bottom_right.y + y_offset,
            bottom_right.z + z_offset,
            // ----
            top_left.x + x_offset,
            top_left.y + y_offset,
            top_left.z + z_offset,
            // ----
            top_right.x + x_offset,
            top_right.y + y_offset,
            top_right.z + z_offset,
        ];
        if backface {
            indices = vec![
                2, 0, 1, //
                1, 3, 2, //
            ];
        } else {
            indices = vec![
                2, 3, 1, //
                1, 0, 2, //
            ];
        }
    }

    let vertices: Vec<f32> = vertices.iter().map(|n| n * VOXEL_SIZE).collect();
    let indices: Vec<i32> = indices;

    // Colors
    const COLOR_CAPACITY: usize = 9;
    const COLOR_VERTEX_SIZE: usize = 3;

    let mut colors;
    {
        colors = Vec::with_capacity(COLOR_CAPACITY);
        let mut i = 0;
        while i < COLOR_CAPACITY {
            match voxel.vf_type {
                VOXEL_TYPE_DEFAULT => {
                    colors.push(1.0);
                    colors.push(0.0);
                    colors.push(0.0);
                }
                VOXEL_TYPE_DIRT => {
                    colors.push(0.23);
                    colors.push(0.168);
                    colors.push(0.086);
                }
                VOXEL_TYPE_GRASS => {
                    colors.push(0.0);
                    colors.push(1.0);
                    colors.push(0.0);
                }
                _ => {
                    colors.push(0.0);
                    colors.push(0.0);
                    colors.push(0.0);
                }
            }

            i += COLOR_VERTEX_SIZE;
        }
    }

    // Normals
    const NORMAL_VERTEX_SIZE: usize = 3;
    let mut normals;
    {
        let mut triangles: Vec<(Vector3<f32>, Vector3<f32>, Vector3<f32>)> = vec![]; // todo; populate triangles
                                                                                     // map triangles
        {
            let num_triangles = indices.len() / VALUES_IN_VERTEX;

            for i in 0..num_triangles {
                let j = i * VALUES_IN_VERTEX;

                let index_1 = j;
                let index_2 = j + 1;
                let index_3 = j + 2;

                // Get start index of each point
                let p1_start = indices[index_1] as usize;
                let p2_start = indices[index_2] as usize;
                let p3_start = indices[index_3] as usize;

                // Assemble vecs of each point
                let p1 = Vector3::<f32>::new(
                    vertices[p1_start],
                    vertices[p1_start + 1],
                    vertices[p1_start + 2],
                );

                let p2 = Vector3::<f32>::new(
                    vertices[p2_start],
                    vertices[p2_start + 1],
                    vertices[p2_start + 2],
                );

                let p3 = Vector3::<f32>::new(
                    vertices[p3_start],
                    vertices[p3_start + 1],
                    vertices[p3_start + 2],
                );

                triangles.push((p1, p2, p3));
            }
        }

        let mapped_normals: Vec<(f32, f32, f32)> = triangles
            .iter()
            .map(|(p1, p2, p3)| calculate_surface_normal_from_triangle(*p1, *p2, *p3))
            .collect();

        normals = Vec::with_capacity(mapped_normals.len() * NORMAL_VERTEX_SIZE);

        mapped_normals
            .iter()
            .for_each(|(normal_x, normal_y, normal_z)| {
                normals.push(*normal_x);
                normals.push(*normal_y);
                normals.push(*normal_z);
            });
    }
    let normals = normals;

    let mesh = Mesh::new(
        VALUES_IN_VERTEX,
        vertices,
        indices,
        COLOR_VERTEX_SIZE,
        colors,
        NORMAL_VERTEX_SIZE,
        normals,
        generated_at_frame,
    );
    return mesh;
}

fn calculate_surface_normal_from_triangle(
    p1: Vector3<f32>,
    p2: Vector3<f32>,
    p3: Vector3<f32>,
) -> (f32, f32, f32) {
    // Referenced: https://www.khronos.org/opengl/wiki/Calculating_a_Surface_Normal

    let u = p2 - p1;
    let v = p3 - p1;

    let normal_x = (u.y * v.z) - (u.z * v.y);
    let normal_y = (u.z * v.x) - (u.x * v.z);
    let normal_z = (u.x * v.y) - (u.y * v.x);

    return (normal_x, normal_y, normal_z);
}

/// Struct used for meshing purposes
#[derive(Debug, Copy, Clone)]
struct VoxelFace {
    pub transparent: bool,
    pub vf_type: u8,
    pub side: usize,
}

impl VoxelFace {
    fn equals(&self, other: &VoxelFace) -> bool {
        return self.transparent == other.transparent && self.vf_type == other.vf_type;
    }
}
