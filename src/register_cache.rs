use crate::prelude::*;

// this just needs to be bigger than the max register we'll see
const REGISTER_COUNT: usize = 256;

#[derive(Clone, Debug)]
pub enum ChannelData {
    ReadRegister(Register, Rc<RefCell<oneshot::Sender<u16>>>),
    RegisterData(Register, u16),
    Shutdown,
}

#[derive(Clone, Debug)]
pub enum Register {
    Hold(u16),
    Input(u16),
}

pub struct RegisterCache {
    channels: Channels,
    hold_register_data: Rc<RefCell<[u16; REGISTER_COUNT]>>,
    input_register_data: Rc<RefCell<[u16; REGISTER_COUNT]>>,
}

impl RegisterCache {
    pub fn new(channels: Channels) -> Self {
        let hold_register_data = Rc::new(RefCell::new([0; REGISTER_COUNT]));
        let input_register_data = Rc::new(RefCell::new([0; REGISTER_COUNT]));

        Self {
            channels,
            hold_register_data,
            input_register_data,
        }
    }

    pub async fn start(&self) -> Result<()> {
        futures::try_join!(self.cache_getter(), self.cache_setter())?;

        Ok(())
    }

    // external helper method to simplify access to the cache, use like so:
    //
    //   RegisterCache::get(&self.channels, 1);
    //
    pub async fn get(channels: &Channels, register: Register) -> u16 {
        let (tx, rx) = oneshot::channel();

        let channel_data = ChannelData::ReadRegister(register, Rc::new(RefCell::new(tx)));
        let _ = channels.read_register_cache.send(channel_data);
        rx.await
            .expect("unexpected error reading from register cache")
    }

    async fn cache_getter(&self) -> Result<()> {
        let mut receiver = self.channels.read_register_cache.subscribe();

        info!("register_cache getter starting");

        while let ChannelData::ReadRegister(register, reply_tx) = receiver.recv().await? {
            match register {
                Register::Hold(r) => {
                    if r < REGISTER_COUNT as u16 {
                        let register_data = self.hold_register_data.borrow();
                        let value = register_data[r as usize];

                        let reply_tx = Rc::try_unwrap(reply_tx).unwrap();
                        let _ = reply_tx.into_inner().send(value);
                    } else {
                        bail!("cannot cache hold register {}, increase REGISTER_COUNT!", r);
                    }
                }
                Register::Input(r) => {
                    if r < REGISTER_COUNT as u16 {
                        let register_data = self.input_register_data.borrow();
                        let value = register_data[r as usize];

                        let reply_tx = Rc::try_unwrap(reply_tx).unwrap();
                        let _ = reply_tx.into_inner().send(value);
                    } else {
                        bail!("cannot cache input register {}, increase REGISTER_COUNT!", r);
                    }
                }
            };
        }

        info!("register_cache getter exiting");

        Ok(())
    }

    async fn cache_setter(&self) -> Result<()> {
        let mut receiver = self.channels.to_register_cache.subscribe();

        info!("register_cache setter starting");

        while let ChannelData::RegisterData(register, value) = receiver.recv().await? {
            // debug!("register_cache setting {:?}={}", register, value);
            match register {
                Register::Hold(r) => {
                    if r < REGISTER_COUNT as u16 {
                        let mut register_data = self.hold_register_data.borrow_mut();
                        register_data[r as usize] = value;
                    } else {
                        bail!("cannot cache hold register {}, increase REGISTER_COUNT!", r);
                    }
                }
                Register::Input(r) => {
                    if r < REGISTER_COUNT as u16 {
                        let mut register_data = self.input_register_data.borrow_mut();
                        register_data[r as usize] = value;
                    } else {
                        bail!("cannot cache input register {}, increase REGISTER_COUNT!", r);
                    }
                }
            };
        }

        info!("register_cache setter exiting");

        Ok(())
    }
}
