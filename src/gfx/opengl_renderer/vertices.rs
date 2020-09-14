pub struct Vertices {
    vertices: Vec<f32>,
    components_per_vertex: Vec<usize>, // The length of each component in the single vertex. E.g. [3,3] could be used for 3 f32's of position info, 3 f32's of color info
}

impl Vertices {
    /// Create a new vertices object
    pub fn new(vertices: Vec<f32>, components_per_vertex: Vec<usize>) -> Self {
        Self {
            vertices: vertices,
            components_per_vertex: components_per_vertex,
        }
    }

    /// Retrieve the list of all vertices
    pub fn vertices(&self) -> &Vec<f32> {
        &self.vertices
    }

    /// Join two sets of vertices. Will panic if strides or components do not match.
    pub fn join(&mut self, other: &Self) -> &Self {
        if self.stride_length() != other.stride_length() {
            panic!("Vertice stride length must be equal!");
        }

        if self.components_per_vertex().len() != other.components_per_vertex().len() {
            panic!("Components per vertex must match!");
        }

        for (i, component) in self.components_per_vertex().iter().enumerate() {
            let other_component = other.components_per_vertex()[i];
            if *component != other_component {
                panic!(
                    "Components do not match! self: {}, other: {}, index: {}",
                    component, other_component, i
                );
            }
        }

        let mut other_verts = other.vertices.clone();

        self.vertices.append(&mut other_verts);

        return self;
    }

    /// The number of indexes within the structure
    pub fn index_length(&self) -> usize {
        self.vertices().len() / self.stride_length()
    }

    /// The length of a single stride of data
    pub fn stride_length(&self) -> usize {
        let mut len = 0;
        for component_len in self.components_per_vertex() {
            len += component_len;
        }

        return len;
    }

    /// The components and their length per vertex
    pub fn components_per_vertex(&self) -> &Vec<usize> {
        &self.components_per_vertex
    }
}
