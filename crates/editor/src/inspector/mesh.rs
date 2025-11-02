use crate::prelude::*;

crate::register_inspectable_readonly!(Mesh, "Mesh");

impl InspectableReadOnly for Mesh {
    fn inspect_readonly(&self, ui: &mut egui::Ui) {
        // Format numbers with thousand separators
        let vertices = format_with_separator(self.vertices.len());
        let indices = format_with_separator(self.indices.len());
        let triangles = format_with_separator(self.indices.len() / 3);

        ui.label(format!("Vertices: {}", vertices));
        ui.label(format!("Indices: {}", indices));
        ui.label(format!("Triangles: {}", triangles));

        // Calculate approximate memory usage
        let vertex_bytes = self.vertices.len() * std::mem::size_of::<Vertex>();
        let index_bytes = self.indices.len() * std::mem::size_of::<Index>();
        let total_bytes = (vertex_bytes + index_bytes) as f32;

        let (size, unit) = if total_bytes >= 1_073_741_824.0 {
            (total_bytes / 1_073_741_824.0, "GB")
        } else if total_bytes >= 1_048_576.0 {
            (total_bytes / 1_048_576.0, "MB")
        } else if total_bytes >= 1024.0 {
            (total_bytes / 1024.0, "KB")
        } else {
            (total_bytes, "B")
        };

        ui.label(format!("Memory: {:.2} {}", size, unit));
    }
}

fn format_with_separator(n: usize) -> String {
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut result = String::new();

    for (i, &byte) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(byte as char);
    }

    result
}
