struct App {
	state: i32
}

impl App {
	fn render(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame){
		if ui.button(format!("Hello from shared lib! State is: {}", self.state)).clicked() {
			self.state += 1;
		}
	}
}

#[no_mangle]
extern "C" fn app_init() -> (*const App, usize) {
	(Box::into_raw(Box::new(App{
		state: 12
	})), core::mem::size_of::<App>())
}

#[no_mangle]
extern "C" fn app_render(ptr: *const App, ui: &mut egui::Ui, frame: &mut eframe::Frame){unsafe{
	let app = ptr as *mut App;
	(*app).render(ui, frame);
}}

