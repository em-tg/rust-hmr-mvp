extern "C" {
	fn dlopen(filename: *const i8, flags: i32) -> *mut u8;
	fn dlclose(handle: *mut u8) -> i32;
	fn dlsym(handle: *mut u8, symbol: *const i8) -> *mut u8;
}

const RTLD_NOW: i32 = 2;

use eframe::egui;

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native("My egui App", native_options, Box::new(|cc| Ok(Box::new(MyEguiApp::new(cc)))));
}

#[derive(Default)]
struct MyEguiApp {
	timer: i64,
	prev_size: usize,
	so_ptr: *mut u8,
	so_data_ptr: *const u8,
	init_fn: Option<extern "C" fn() -> (*const u8, usize)>,
	render_fn: Option<extern "C" fn(ptr: *const u8, ui: &mut egui::Ui, frame: &mut eframe::Frame)>,
}

impl MyEguiApp {
	fn new(cc: &eframe::CreationContext<'_>) -> Self {
		// Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_global_style.
		// Restore app state using cc.storage (requires the "persistence" feature).
		// Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
		// for e.g. egui::PaintCallback.
		let mut ret = Self::default();
		ret.reload(true);
		ret
	}


	fn rebuild(&mut self) -> Option<()>{
		std::process::Command::new("cargo")
			.args(["b"])
			.output()
			.ok()?;

		Some(())
	}

	fn reload(&mut self, force: bool){unsafe{
		if !self.so_ptr.is_null() {
			let ret = dlclose(self.so_ptr);
			assert!(ret != -1);
			self.so_ptr = core::ptr::null_mut();
		}

		let new_ptr = dlopen(c"libhmr.so".as_ptr(), RTLD_NOW);
		assert!(!new_ptr.is_null());

		self.so_ptr = new_ptr;
		let init_fn = dlsym(self.so_ptr, c"app_init".as_ptr());
		assert!(!init_fn.is_null());
		let render_fn = dlsym(self.so_ptr, c"app_render".as_ptr());
		assert!(!render_fn.is_null());

		self.init_fn = Some(core::mem::transmute(init_fn));
		self.render_fn = Some(core::mem::transmute(render_fn));

		let (data_ptr, data_size) = (self.init_fn.unwrap())();
		assert!(!data_ptr.is_null());
		if force || data_size != self.prev_size {
			self.so_data_ptr = data_ptr;
			self.prev_size = data_size;
		}
	}}
}

impl eframe::App for MyEguiApp {
	fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
		egui::CentralPanel::default().show_inside(ui, |ui| {
			self.timer += 1;
			if self.timer % 60 == 0 {
				if let Some(()) = self.rebuild() { // TODO: probably want to do this off-thread :P
					self.reload(false);
				}
			}

			(self.render_fn.unwrap())(self.so_data_ptr, ui, frame);

			ui.ctx().request_repaint(); // For testing purposes
		});
	}
}
