use winit::event_loop::EventLoop;

pub fn os_window() -> Result<EventLoop<()>, winit::error::EventLoopError> {
    let event_loop = EventLoop::new()?;

    todo!()
}
