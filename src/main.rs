use eframe::egui;
use egui_plot::{Line, Plot, PlotPoints, Points};
use glam::{Mat3, Vec3};
use meval::{self, Expr};

fn main() -> Result<(), eframe::Error> {
    // Use x11 feature to prevent Wayland-related issues
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_min_inner_size([400.0, 300.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Rust 3D Grapher",
        options,
        Box::new(|cc| Ok(Box::new(RustGrapherApp::new(cc)))),
    )
}

#[derive(Clone)]
struct Vertex3D {
    position: Vec3,
    color: egui::Color32,
}

struct Surface {
    vertices: Vec<Vec<Vertex3D>>,
    range: f64,
    resolution: usize,
}

impl Surface {
    fn new(resolution: usize, range: f64) -> Self {
        let vertices = vec![vec![Vertex3D {
            position: Vec3::ZERO,
            color: egui::Color32::WHITE,
        }; resolution + 1]; resolution + 1];
        
        Self {
            vertices,
            range,
            resolution,
        }
    }
    
    fn generate_from_expression<F>(&mut self, expr_fn: F) 
    where 
        F: Fn(f64, f64) -> Option<f64>
    {
        let step = (2.0 * self.range) / self.resolution as f64;
        
        let mut min_z = f64::MAX;
        let mut max_z = f64::MIN;
        
        // First pass: evaluate all points and find min/max z values for color mapping
        for i in 0..=self.resolution {
            let y = -self.range + i as f64 * step;
            
            for j in 0..=self.resolution {
                let x = -self.range + j as f64 * step;
                
                if let Some(z) = expr_fn(x, y) {
                    let position = Vec3::new(x as f32, y as f32, z as f32);
                    self.vertices[i][j].position = position;
                    
                    min_z = min_z.min(z);
                    max_z = max_z.max(z);
                } else {
                    // Set a special value for invalid points
                    self.vertices[i][j].position = Vec3::new(x as f32, y as f32, 0.0);
                }
            }
        }
        
        // Normalize z-range if it's not degenerate
        let z_range = if max_z > min_z { max_z - min_z } else { 1.0 };
        
        // Second pass: set colors based on normalized z value
        for i in 0..=self.resolution {
            for j in 0..=self.resolution {
                let z = self.vertices[i][j].position.z as f64;
                let normalized_z = if max_z > min_z {
                    (z - min_z) / z_range
                } else {
                    0.5 // Default to middle color for flat surfaces
                };
                
                // Set color based on the normalized z value (blue to red gradient)
                self.vertices[i][j].color = color_from_height(normalized_z);
            }
        }
    }
    
    // Project points to 2D with rotation and zoom
    fn project_points(&self, rotation_matrix: Mat3, zoom: f32) -> (Vec<[f64; 2]>, Vec<egui::Color32>) {
        let mut positions = Vec::new();
        let mut colors = Vec::new();
        
        for row in &self.vertices {
            for vertex in row {
                // Apply rotation and scaling
                let rotated = rotation_matrix * vertex.position * zoom;
                
                // Project to 2D (simple orthographic projection)
                let projected_x = rotated.x as f64;
                let projected_y = rotated.z as f64; // Use z as y for top-down view
                
                positions.push([projected_x, projected_y]);
                colors.push(vertex.color);
            }
        }
        
        (positions, colors)
    }
    
    // Get wireframe lines for the surface (horizontal and vertical grid lines)
    fn get_wireframe_lines(&self, rotation_matrix: Mat3, zoom: f32) -> Vec<Line> {
        let mut lines = Vec::new();
        
        // Horizontal grid lines (constant y)
        for i in 0..=self.resolution {
            let mut line_points = Vec::new();
            
            for j in 0..=self.resolution {
                let rotated = rotation_matrix * self.vertices[i][j].position * zoom;
                line_points.push([rotated.x as f64, rotated.z as f64]);
            }
            
            lines.push(Line::new(format!("row_{}", i), PlotPoints::new(line_points))
                .color(egui::Color32::from_gray(180))
                .width(1.0));
        }
        
        // Vertical grid lines (constant x)
        for j in 0..=self.resolution {
            let mut line_points = Vec::new();
            
            for i in 0..=self.resolution {
                let rotated = rotation_matrix * self.vertices[i][j].position * zoom;
                line_points.push([rotated.x as f64, rotated.z as f64]);
            }
            
            lines.push(Line::new(format!("col_{}", j), PlotPoints::new(line_points))
                .color(egui::Color32::from_gray(180))
                .width(1.0));
        }
        
        lines
    }
}

// Helper function to convert height to color (blue to red gradient)
fn color_from_height(height: f64) -> egui::Color32 {
    if height < 0.0 || height > 1.0 {
        return egui::Color32::BLACK; // Invalid range
    }
    
    // Simple blue to red gradient through green
    if height < 0.5 {
        // Blue to green (0.0 -> 0.5)
        let t = height * 2.0;
        let r = (t * 255.0) as u8;
        let g = (t * 255.0) as u8;
        let b = ((1.0 - t) * 255.0) as u8;
        egui::Color32::from_rgb(r, g, b)
    } else {
        // Green to red (0.5 -> 1.0)
        let t = (height - 0.5) * 2.0;
        let r = 255;
        let g = ((1.0 - t) * 255.0) as u8;
        let b = 0;
        egui::Color32::from_rgb(r, g, b)
    }
}

// Helper function to create rotation matrix from euler angles
fn rotation_matrix(x_deg: f32, y_deg: f32, z_deg: f32) -> Mat3 {
    let x_rad = x_deg.to_radians();
    let y_rad = y_deg.to_radians();
    let z_rad = z_deg.to_radians();
    
    let rot_x = Mat3::from_rotation_x(x_rad);
    let rot_y = Mat3::from_rotation_y(y_rad);
    let rot_z = Mat3::from_rotation_z(z_rad);
    
    // Combine rotations: first Z, then Y, then X
    rot_x * rot_y * rot_z
}

struct RustGrapherApp {
    expression: String,
    expression_obj: Option<Expr>,
    rotation_x: f32,
    rotation_y: f32,
    rotation_z: f32,
    zoom: f32,
    grid_resolution: usize,
    range: f64,
    error_message: Option<String>,
    example_expressions: Vec<String>,
    surface: Surface,
    show_wireframe: bool,
    show_points: bool,
    auto_rotate: bool,
}

impl RustGrapherApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let resolution = 20;
        let range = 3.0;
        let surface = Surface::new(resolution, range);
        
        let mut app = Self {
            expression: "sin(x) * cos(y)".to_string(),
            expression_obj: None,
            rotation_x: 30.0,
            rotation_y: 30.0,
            rotation_z: 0.0,
            zoom: 0.8,
            grid_resolution: resolution,
            range,
            error_message: None,
            example_expressions: vec![
                "sin(x) * cos(y)".to_string(),
                "x^2 + y^2".to_string(),
                "sin(sqrt(x^2 + y^2))".to_string(),
                "exp(-(x^2 + y^2))".to_string(),
                "sin(x*y)".to_string(),
            ],
            surface,
            show_wireframe: true,
            show_points: true,
            auto_rotate: false,
        };
        
        // Initialize with default expression
        app.compile_expression();
        
        app
    }

    fn compile_expression(&mut self) {
        match self.expression.parse::<Expr>() {
            Ok(expr) => {
                self.expression_obj = Some(expr);
                self.error_message = None;
                
                // Generate surface data
                let expr_obj = self.expression_obj.clone().unwrap();
                self.update_surface_data(expr_obj);
            }
            Err(err) => {
                self.error_message = Some(format!("Error parsing expression: {}", err));
                self.expression_obj = None;
            }
        }
    }
    
    fn update_surface_data(&mut self, expr: Expr) {
        // Create a new surface with current settings
        self.surface = Surface::new(self.grid_resolution, self.range);
        
        // Generate surface data from expression
        self.surface.generate_from_expression(|x, y| {
            let mut context = meval::Context::new();
            context.var("x", x);
            context.var("y", y);
            
            match expr.eval_with_context(context) {
                Ok(z) if z.is_finite() => Some(z),
                _ => None,
            }
        });
    }
}

impl eframe::App for RustGrapherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Auto-rotation if enabled
        if self.auto_rotate {
            self.rotation_y = (self.rotation_y + 0.5) % 360.0;
            ctx.request_repaint();
        }
        
        // Top panel with app title
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.heading("Rust 3D Grapher");
                
                // Dark/Light mode toggle
                ui.menu_button("Theme", |ui| {
                    if ui.button("Light").clicked() {
                        ctx.set_visuals(egui::Visuals::light());
                    }
                    if ui.button("Dark").clicked() {
                        ctx.set_visuals(egui::Visuals::dark());
                    }
                });
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to("Source", "https://github.com/yourusername/rust_grapher");
                });
            });
        });

        // Bottom panel with input controls
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Expression:");
                
                let text_edit = ui.text_edit_singleline(&mut self.expression);
                
                if text_edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    self.compile_expression();
                }
                
                if ui.button("Graph").clicked() {
                    self.compile_expression();
                }
                
                // Handle example selection without borrowing self twice
                let mut selected_example = None;
                egui::ComboBox::from_label("Examples")
                    .selected_text("Select an example...")
                    .show_ui(ui, |ui| {
                        for example in &self.example_expressions {
                            if ui.selectable_label(false, example).clicked() {
                                selected_example = Some(example.clone());
                            }
                        }
                    });
                
                // Apply the selected example outside of the ComboBox closure
                if let Some(example) = selected_example {
                    self.expression = example;
                    self.compile_expression();
                }
            });
            
            if let Some(error) = &self.error_message {
                ui.colored_label(egui::Color32::RED, error);
            }
        });

        // Left panel with controls
        egui::SidePanel::left("side_panel").show(ctx, |ui| {
            ui.heading("Controls");
            
            ui.checkbox(&mut self.auto_rotate, "Auto Rotate");
            
            ui.add(egui::Slider::new(&mut self.rotation_x, 0.0..=360.0).text("Rotation X"));
            ui.add(egui::Slider::new(&mut self.rotation_y, 0.0..=360.0).text("Rotation Y"));
            ui.add(egui::Slider::new(&mut self.rotation_z, 0.0..=360.0).text("Rotation Z"));
            ui.add(egui::Slider::new(&mut self.zoom, 0.1..=5.0).text("Zoom"));
            
            ui.add_space(10.0);
            ui.heading("Graph Settings");
            
            ui.checkbox(&mut self.show_wireframe, "Show Wireframe");
            ui.checkbox(&mut self.show_points, "Show Points");
            
            if ui.add(egui::Slider::new(&mut self.grid_resolution, 5..=50).text("Resolution")).changed() {
                if let Some(expr) = &self.expression_obj {
                    self.update_surface_data(expr.clone());
                }
            }
            
            if ui.add(egui::Slider::new(&mut self.range, 0.5..=10.0).text("Range")).changed() {
                if let Some(expr) = &self.expression_obj {
                    self.update_surface_data(expr.clone());
                }
            }
            
            ui.add_space(10.0);
            ui.heading("Function Info");
            
            ui.label("Supported Functions:");
            ui.label("- Basic: +, -, *, /, ^, %");
            ui.label("- Trigonometric: sin, cos, tan");
            ui.label("- Inverse trig: asin, acos, atan");
            ui.label("- Hyperbolic: sinh, cosh, tanh");
            ui.label("- Other: exp, ln, log, abs, sqrt");
            
            ui.add_space(10.0);
            ui.label("Constants: π (pi), e");
        });

        // Main central area for 3D plot
        egui::CentralPanel::default().show(ctx, |ui| {
            // Ensure we have a valid expression before trying to plot
            if self.expression_obj.is_none() && self.error_message.is_none() {
                self.compile_expression();
            }
            
            // Create rotation matrix from Euler angles
            let rotation = rotation_matrix(self.rotation_x, self.rotation_y, self.rotation_z);
            
            // Project points for plotting
            let (positions, colors) = self.surface.project_points(rotation, self.zoom);
            
            // Main plot area
            Plot::new("3d_plot")
                .data_aspect(1.0)
                .show(ui, |plot_ui| {
                    // Draw wireframe if enabled
                    if self.show_wireframe {
                        let lines = self.surface.get_wireframe_lines(rotation, self.zoom);
                        for line in lines {
                            plot_ui.line(line);
                        }
                    }
                    
                    // Draw surface points if enabled
                    if self.show_points {
                        // Simple approach: Create points with different colors
                        for (idx, pos) in positions.iter().enumerate() {
                            if idx < colors.len() {
                                let color = colors[idx];
                                let points = Points::new(format!("point_{}", idx), PlotPoints::new(vec![*pos]))
                                    .color(color)
                                    .radius(3.0);
                                plot_ui.points(points);
                            }
                        }
                    }
                });
            
            // Help text
            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                ui.label("Drag to pan, scroll to zoom the plot view");
            });
        });

        // Request a repaint if we're animating
        if self.auto_rotate {
            ctx.request_repaint();
        }
    }
}