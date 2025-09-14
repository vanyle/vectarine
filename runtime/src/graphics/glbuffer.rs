use std::sync::Arc;

use glow::HasContext;

use crate::graphics::gltypes::DataLayout;

/// Represents a buffer stored in a GPU that can be drawn.
/// We don"t store the associate CPU data here.
pub struct GpuVertexData {
    // vbo + vao
    pub vbo: glow::NativeBuffer,
    pub vao: glow::NativeVertexArray,
    pub ebo: glow::NativeBuffer,
    pub layout: DataLayout,
    pub drawn_point_count: usize,
    pub buffer_row_count: usize,
    gl: Arc<glow::Context>,
}

/// Give a hint to the driver on how you intent to use the data.
/// See https://docs.gl/es3/glBufferData
pub enum BufferUsageHint {
    StaticDraw,
    StreamDraw,
    DynamicDraw,
}

impl BufferUsageHint {
    fn to_gl_enum(&self) -> u32 {
        match self {
            BufferUsageHint::StaticDraw => glow::STATIC_DRAW,
            BufferUsageHint::StreamDraw => glow::STREAM_DRAW,
            BufferUsageHint::DynamicDraw => glow::DYNAMIC_DRAW,
        }
    }
}

impl GpuVertexData {
    pub fn new(gl: &Arc<glow::Context>) -> Self {
        let vao = unsafe { gl.create_vertex_array().unwrap() };
        let vbo = unsafe { gl.create_buffer().unwrap() };
        let ebo = unsafe { gl.create_buffer().unwrap() };

        Self {
            vbo,
            vao,
            ebo,
            layout: DataLayout::new(),
            drawn_point_count: 0,
            buffer_row_count: 0,
            gl: gl.clone(),
        }
    }

    pub fn set_data<T: Copy>(
        &mut self,
        vertex_data: &[T],
        index_data: &[u32],
    ) -> Result<(), String> {
        self.set_data_with_usage(vertex_data, index_data, BufferUsageHint::StaticDraw)
    }

    pub fn set_data_with_usage<T: Copy>(
        &mut self,
        vertex_data: &[T],
        index_data: &[u32],
        usage: BufferUsageHint,
    ) -> Result<(), String> {
        if self.layout.fields.is_empty() {
            return Err("You must apply a layout before setting data!".to_string());
        }
        self.drawn_point_count = index_data.len();
        let vertex_raw_data;
        let vertex_data_byte_count = std::mem::size_of_val(vertex_data);
        unsafe {
            vertex_raw_data = std::slice::from_raw_parts(
                vertex_data.as_ptr() as *const u8,
                vertex_data_byte_count,
            );
        }

        let e = self.layout.is_sound(vertex_raw_data, index_data, 0);
        if let Some(e) = e {
            return Err(format!(
                "The provided data is not sound for the current layout: {e}"
            ));
        }

        let stride = self.layout.stride();
        let point_count = vertex_data_byte_count / stride;
        self.buffer_row_count = point_count;

        unsafe {
            let gl = self.gl.as_ref();
            gl.bind_vertex_array(Some(self.vao)); // might not be needed, I need to check.

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));

            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, vertex_raw_data, usage.to_gl_enum());

            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
            gl.buffer_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                convert_u32_to_u8(index_data),
                usage.to_gl_enum(),
            );

            // We unbind these after unbinding the VAO as the VAO remembers to what buffer it should bound!
            // Not needed, but we do it for safety.
            gl.bind_vertex_array(None);
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        }
        Ok(())
    }

    /// Transfer the layout information (how this data is supposed to be understood by the gpu)
    /// from the CPU to the CPU
    pub fn apply_layout(&mut self, layout: DataLayout) {
        self.layout = layout;
        unsafe {
            let gl = self.gl.as_ref();
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));

            let stride = self.layout.stride() as i32;
            let mut offset = 0;

            for (i, (_name, gl_type, _)) in self.layout.fields.iter().enumerate() {
                let size = gl_type.size_in_bytes() as i32;
                let count = gl_type.component_count() as i32;
                let gl_type_enum = gl_type.to_gl_subtype();
                gl.vertex_attrib_pointer_f32(i as u32, count, gl_type_enum, false, stride, offset);
                gl.enable_vertex_attrib_array(i as u32);
                offset += size;
            }

            // Not needed, but we do it for safety.
            gl.bind_vertex_array(None);
            // We unbind these after unbinding the VAO as the VAO remembers to what buffer it should bound!
            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, None);
        }
    }

    pub fn bind_for_drawing(&self) {
        unsafe {
            let gl = self.gl.as_ref();
            gl.bind_vertex_array(Some(self.vao));
        }
    }
}

impl std::fmt::Debug for GpuVertexData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "GpuVertexData with {} rows for {} points",
            self.buffer_row_count, self.drawn_point_count
        )?;
        Ok(())
    }
}

impl Drop for GpuVertexData {
    fn drop(&mut self) {
        let gl = self.gl.as_ref();
        unsafe {
            gl.delete_vertex_array(self.vao);
            gl.delete_buffer(self.vbo);
            gl.delete_buffer(self.ebo);
        }
    }
}

pub fn convert_u32_to_u8(data: &[u32]) -> &[u8] {
    let len = 4 * data.len();
    let ptr = data.as_ptr() as *const u8;
    unsafe { std::slice::from_raw_parts(ptr, len) }
}

/// A buffer that contains data which may or may not be in the GPU.
/// The data in the buffer is always sound with regards to the layout.
#[derive(Debug)]
pub struct SharedGPUCPUBuffer {
    cpu_vertex_data: Vec<u8>,
    cpu_index_data: Vec<u32>,
    gpu_buffer: Option<GpuVertexData>,
    gpu_up_to_date: bool,
    layout: DataLayout,
}

impl SharedGPUCPUBuffer {
    pub fn new(layout: DataLayout) -> Self {
        Self {
            cpu_vertex_data: Vec::new(),
            cpu_index_data: Vec::new(),
            gpu_buffer: None,
            gpu_up_to_date: false,
            layout,
        }
    }
    pub fn from_raw_data(layout: DataLayout, vertex_data: Vec<u8>, index_data: Vec<u32>) -> Self {
        #![cfg(debug_assertions)]
        if let Some(e) = layout.is_sound(&vertex_data, &index_data, 0) {
            panic!("The provided data is not sound for the provided layout: {e}")
        }
        Self {
            cpu_vertex_data: vertex_data,
            cpu_index_data: index_data,
            gpu_buffer: None,
            gpu_up_to_date: false,
            layout,
        }
    }

    pub fn from_data<T: Copy>(layout: DataLayout, vertex_data: &[T], index_data: &[u32]) -> Self {
        let vertex_data_byte_count = std::mem::size_of_val(vertex_data);
        let vertex_raw_data;
        unsafe {
            vertex_raw_data = std::slice::from_raw_parts(
                vertex_data.as_ptr() as *const u8,
                vertex_data_byte_count,
            );
        }
        Self::from_raw_data(layout, vertex_raw_data.to_vec(), index_data.to_vec())
    }

    /// Append data to the buffer. Performs index renumbering.
    pub fn append(&mut self, data: &[u8], indices: &[u32]) {
        #![cfg(debug_assertions)]
        if let Some(e) = self.layout.is_sound(data, indices, 0) {
            panic!("The provided data is not sound for the current layout: {e}")
        }
        let vertex_offset = self.cpu_vertex_data.len() / self.layout.stride();
        let renumbered_indices = indices.iter().map(|i| i + vertex_offset as u32);
        self.cpu_vertex_data.extend_from_slice(data);
        self.cpu_index_data.extend(renumbered_indices);
        self.gpu_up_to_date = false;
    }

    pub fn append_from<T: Copy>(&mut self, data: &[T], indices: &[u32]) {
        let vertex_data_byte_count = std::mem::size_of_val(data);
        let vertex_raw_data;
        unsafe {
            vertex_raw_data =
                std::slice::from_raw_parts(data.as_ptr() as *const u8, vertex_data_byte_count);
        }
        self.append(vertex_raw_data, indices);
    }

    pub fn send_to_gpu(&mut self, gl: &Arc<glow::Context>) -> &GpuVertexData {
        self.send_to_gpu_with_usage(gl, BufferUsageHint::StaticDraw)
    }

    pub fn send_to_gpu_with_usage(
        &mut self,
        gl: &Arc<glow::Context>,
        usage_hint: BufferUsageHint,
    ) -> &GpuVertexData {
        if self.gpu_up_to_date {
            return self.gpu_buffer.as_ref().unwrap();
        }
        let mut gpu_buffer = GpuVertexData::new(gl);
        gpu_buffer.apply_layout(self.layout.clone());
        // We know set_data only performs a soundness check, so we can safely unwrap.
        gpu_buffer
            .set_data_with_usage(&self.cpu_vertex_data, &self.cpu_index_data, usage_hint)
            .unwrap();
        self.gpu_buffer = Some(gpu_buffer);
        self.gpu_up_to_date = true;
        self.gpu_buffer.as_ref().unwrap()
    }

    pub fn is_on_gpu(&self) -> bool {
        self.gpu_buffer.is_some()
    }

    pub fn gpu_up_to_date(&self) -> bool {
        self.gpu_up_to_date
    }

    pub fn gpu_buffer(&self) -> Option<&GpuVertexData> {
        self.gpu_buffer.as_ref()
    }

    pub fn clear_cpu_data(&mut self) {
        // Note: no data is always sound.
        self.cpu_vertex_data.clear();
        self.cpu_index_data.clear();
    }

    pub fn clear(&mut self) {
        self.cpu_vertex_data.clear();
        self.cpu_index_data.clear();
        self.gpu_buffer = None;
        self.gpu_up_to_date = false;
    }
}
