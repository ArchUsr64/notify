use crate::app::App;
use log::{log_enabled, Level};
use smithay_client_toolkit::{
	compositor::{CompositorHandler, CompositorState},
	delegate_compositor, delegate_keyboard, delegate_layer, delegate_output, delegate_pointer,
	delegate_registry, delegate_seat, delegate_shm,
	output::{OutputHandler, OutputState},
	registry::{ProvidesRegistryState, RegistryState},
	registry_handlers,
	seat::{
		keyboard::{KeyEvent, KeyboardHandler, Keysym, Modifiers},
		pointer::{PointerEvent, PointerHandler},
		Capability, SeatHandler, SeatState,
	},
	shell::{
		wlr_layer::{
			KeyboardInteractivity, Layer, LayerShell, LayerShellHandler, LayerSurface,
			LayerSurfaceConfigure,
		},
		WaylandSurface,
	},
	shm::{slot::SlotPool, Shm, ShmHandler},
};
use wayland_client::{
	globals::registry_queue_init,
	protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
	Connection, EventQueue, QueueHandle,
};

/// Manages interfacing with Wayland
pub struct Window {
	registry_state: RegistryState,
	seat_state: SeatState,
	output_state: OutputState,
	shm: Shm,
	first_configure: bool,
	pool: SlotPool,
	width: u32,
	height: u32,
	layer: LayerSurface,
	keyboard: Option<wl_keyboard::WlKeyboard>,
	modifiers: Modifiers,
	pointer: Option<wl_pointer::WlPointer>,
	// Can't be a generic since delegate_* macros require 'static lifetime
	pub app: App,
}

impl Window {
	pub fn new(width: u32, height: u32, app: App) -> (Self, EventQueue<Self>) {
		let conn = Connection::connect_to_env().unwrap();
		let (globals, event_queue) = registry_queue_init(&conn).unwrap();
		let qh: QueueHandle<Self> = event_queue.handle();

		let compositor =
			CompositorState::bind(&globals, &qh).expect("wl_compositor is not available");

		let layer_shell = LayerShell::bind(&globals, &qh).expect("layer shell is not available");

		let shm = Shm::bind(&globals, &qh).expect("wl_shm is not available");

		let surface = compositor.create_surface(&qh);

		let layer =
			layer_shell.create_layer_surface(&qh, surface, Layer::Overlay, Some("notify"), None);
		layer.set_keyboard_interactivity(KeyboardInteractivity::OnDemand);
		layer.set_size(width, height);
		layer.commit();

		let pool = SlotPool::new((width * height * 4) as usize, &shm)
			.expect("Failed to create memory pool");

		(
			Self {
				registry_state: RegistryState::new(&globals),
				seat_state: SeatState::new(&globals, &qh),
				output_state: OutputState::new(&globals, &qh),
				shm,
				first_configure: true,
				pool,
				width,
				height,
				layer,
				keyboard: None,
				// TODO: Handle the case when modifiers are already activated
				modifiers: Modifiers::default(),
				pointer: None,
				app,
			},
			event_queue,
		)
	}
}
impl CompositorHandler for Window {
	fn scale_factor_changed(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_surface: &wl_surface::WlSurface,
		_new_factor: i32,
	) {
		// Not needed for this example.
	}

	fn transform_changed(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_surface: &wl_surface::WlSurface,
		_new_transform: wl_output::Transform,
	) {
		// Not needed for this example.
	}

	fn frame(
		&mut self,
		_conn: &Connection,
		qh: &QueueHandle<Self>,
		_surface: &wl_surface::WlSurface,
		_time: u32,
	) {
		self.draw(qh);
	}
}

impl OutputHandler for Window {
	fn output_state(&mut self) -> &mut OutputState {
		&mut self.output_state
	}

	fn new_output(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_output: wl_output::WlOutput,
	) {
	}

	fn update_output(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_output: wl_output::WlOutput,
	) {
	}

	fn output_destroyed(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_output: wl_output::WlOutput,
	) {
	}
}

impl LayerShellHandler for Window {
	fn closed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _layer: &LayerSurface) {
		self.app.exit();
	}

	fn configure(
		&mut self,
		_conn: &Connection,
		qh: &QueueHandle<Self>,
		_layer: &LayerSurface,
		configure: LayerSurfaceConfigure,
		_serial: u32,
	) {
		if configure.new_size.0 == 0 || configure.new_size.1 == 0 {
			self.width = self.width;
			self.height = self.height;
		} else {
			self.width = configure.new_size.0;
			self.height = configure.new_size.1;
		}

		// Initiate the first draw.
		if self.first_configure {
			self.first_configure = false;
			self.draw(qh);
		}
	}
}

impl SeatHandler for Window {
	fn seat_state(&mut self) -> &mut SeatState {
		&mut self.seat_state
	}

	fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

	fn new_capability(
		&mut self,
		_conn: &Connection,
		qh: &QueueHandle<Self>,
		seat: wl_seat::WlSeat,
		capability: Capability,
	) {
		if capability == Capability::Keyboard && self.keyboard.is_none() {
			let keyboard = self
				.seat_state
				.get_keyboard(qh, &seat, None)
				.expect("Failed to create keyboard");
			self.keyboard = Some(keyboard);
		}

		if capability == Capability::Pointer && self.pointer.is_none() {
			let pointer = self
				.seat_state
				.get_pointer(qh, &seat)
				.expect("Failed to create pointer");
			self.pointer = Some(pointer);
		}
	}

	fn remove_capability(
		&mut self,
		_conn: &Connection,
		_: &QueueHandle<Self>,
		_: wl_seat::WlSeat,
		capability: Capability,
	) {
		if capability == Capability::Keyboard && self.keyboard.is_some() {
			self.keyboard.take().unwrap().release();
		}

		if capability == Capability::Pointer && self.pointer.is_some() {
			self.pointer.take().unwrap().release();
		}
	}

	fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl KeyboardHandler for Window {
	fn enter(
		&mut self,
		_: &Connection,
		_: &QueueHandle<Self>,
		_: &wl_keyboard::WlKeyboard,
		surface: &wl_surface::WlSurface,
		_: u32,
		_: &[u32],
		_: &[Keysym],
	) {
	}

	fn leave(
		&mut self,
		_: &Connection,
		_: &QueueHandle<Self>,
		_: &wl_keyboard::WlKeyboard,
		surface: &wl_surface::WlSurface,
		_: u32,
	) {
	}

	fn press_key(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_: &wl_keyboard::WlKeyboard,
		_: u32,
		event: KeyEvent,
	) {
	}

	fn release_key(
		&mut self,
		_: &Connection,
		_: &QueueHandle<Self>,
		_: &wl_keyboard::WlKeyboard,
		_: u32,
		_: KeyEvent,
	) {
	}

	fn update_modifiers(
		&mut self,
		_: &Connection,
		_: &QueueHandle<Self>,
		_: &wl_keyboard::WlKeyboard,
		_serial: u32,
		modifiers: Modifiers,
	) {
		self.modifiers = modifiers;
	}
}

impl PointerHandler for Window {
	fn pointer_frame(
		&mut self,
		_conn: &Connection,
		_qh: &QueueHandle<Self>,
		_pointer: &wl_pointer::WlPointer,
		_events: &[PointerEvent],
	) {
		// Do nothing with mouse events
	}
}

impl ShmHandler for Window {
	fn shm_state(&mut self) -> &mut Shm {
		&mut self.shm
	}
}

impl Window {
	pub fn draw(&mut self, qh: &QueueHandle<Self>) {
		let width = self.width;
		let height = self.height;
		let stride = self.width as i32 * 4;
		let border_color = 0xff;

		let (buffer, canvas) = self
			.pool
			.create_buffer(
				width as i32,
				height as i32,
				stride,
				wl_shm::Format::Argb8888,
			)
			.expect("create buffer");

		(0..self.width).for_each(|i| {
			let index = 4 * i as usize;
			canvas[index] = border_color;
			canvas[index + 1] = border_color;
			canvas[index + 2] = border_color;
			canvas[index + 3] = 0xff;
		});

		self.app
			.draw(&mut canvas[4 * self.width as usize..], width, height);

		(0..self.width).for_each(|i| {
			let index = canvas.len() - 4 * i as usize - 1;
			canvas[index - 3] = border_color;
			canvas[index - 2] = border_color;
			canvas[index - 1] = border_color;
			canvas[index] = 0xff;
		});

		// Damage the entire window
		self.layer
			.wl_surface()
			.damage_buffer(0, 0, width as i32, height as i32);

		// Request our next frame
		self.layer
			.wl_surface()
			.frame(qh, self.layer.wl_surface().clone());

		// Attach and commit to present.
		buffer
			.attach_to(self.layer.wl_surface())
			.expect("buffer attach");
		self.layer.commit();

		// TODO save and reuse buffer when the window size is unchanged.  This is especially
		// useful if you do damage tracking, since you don't need to redraw the undamaged parts
		// of the canvas.
	}
}

delegate_compositor!(Window);
delegate_output!(Window);
delegate_shm!(Window);

delegate_seat!(Window);
delegate_keyboard!(Window);
delegate_pointer!(Window);

delegate_layer!(Window);

delegate_registry!(Window);

impl ProvidesRegistryState for Window {
	fn registry(&mut self) -> &mut RegistryState {
		&mut self.registry_state
	}
	registry_handlers![OutputState, SeatState];
}
