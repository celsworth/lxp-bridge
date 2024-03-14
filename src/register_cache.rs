use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum ChannelData {
    ReadRegister(Register, Rc<RefCell<oneshot::Sender<u16>>>),
    ReadAllRegisters(AllRegisters, Rc<RefCell<oneshot::Sender<Cache>>>),
    RegisterData(Register, u16),
    ClearInputRegisters,
    Shutdown,
}

#[derive(Clone, Debug)]
pub enum Register {
    Hold(u16),
    Input(u16),
}

#[derive(Clone, Debug)]
pub enum AllRegisters {
    Hold,
    Input,
}

type Cache = HashMap<u16, u16>;

pub struct RegisterCache {
    channels: Channels,
    hold_register_data: Rc<RefCell<Cache>>,
    input_register_data: Rc<RefCell<Cache>>,
}

impl RegisterCache {
    pub fn new(channels: Channels) -> Self {
        let hold_register_data = Rc::new(RefCell::new(Cache::with_capacity(256)));
        let input_register_data = Rc::new(RefCell::new(Cache::with_capacity(256)));

        Self {
            channels,
            hold_register_data,
            input_register_data,
        }
    }

    pub async fn start(&self) -> Result<()> {
        futures::try_join!(self.runner())?;

        Ok(())
    }

    // external helper method to simplify access to the cache, use like so:
    //
    //   RegisterCache::get(&self.channels, 1);
    //
    pub async fn get(channels: &Channels, register: Register) -> u16 {
        let (tx, rx) = oneshot::channel();

        let channel_data = ChannelData::ReadRegister(register, Rc::new(RefCell::new(tx)));
        let _ = channels.register_cache.send(channel_data);
        rx.await
            .expect("unexpected error reading from register cache")
    }

    async fn runner(&self) -> Result<()> {
        let mut receiver = self.channels.register_cache.subscribe();

        info!("register_cache runner starting");

        loop {
            match receiver.recv().await? {
                ChannelData::RegisterData(register, value) => {
                    // debug!("register_cache setting {:?}={}", register, value);
                    match register {
                        Register::Hold(r) => {
                            let mut register_data = self.hold_register_data.borrow_mut();
                            register_data.insert(r, value);
                        }
                        Register::Input(r) => {
                            let mut register_data = self.input_register_data.borrow_mut();
                            register_data.insert(r, value);
                        }
                    };
                }

                ChannelData::ClearInputRegisters => {
                    // not used yet
                    let mut register_data = self.input_register_data.borrow_mut();
                    register_data.clear();
                }

                ChannelData::ReadAllRegisters(register_type, reply_tx) => match register_type {
                    AllRegisters::Hold => {
                        let register_data = self.hold_register_data.borrow().clone();
                        let reply_tx = Rc::try_unwrap(reply_tx).unwrap();
                        let _ = reply_tx.into_inner().send(register_data);
                    }
                    AllRegisters::Input => {
                        let register_data = self.input_register_data.borrow().clone();
                        let reply_tx = Rc::try_unwrap(reply_tx).unwrap();
                        let _ = reply_tx.into_inner().send(register_data);
                    }
                },

                ChannelData::ReadRegister(register, reply_tx) => {
                    match register {
                        Register::Hold(r) => {
                            let register_data = self.hold_register_data.borrow();
                            let value = register_data.get(&r).cloned().unwrap_or(0);

                            let reply_tx = Rc::try_unwrap(reply_tx).unwrap();
                            let _ = reply_tx.into_inner().send(value);
                        }
                        Register::Input(r) => {
                            let register_data = self.input_register_data.borrow();
                            let value = register_data.get(&r).cloned().unwrap_or(0);

                            let reply_tx = Rc::try_unwrap(reply_tx).unwrap();
                            let _ = reply_tx.into_inner().send(value);
                        }
                    };
                }

                ChannelData::Shutdown => break,
            }
        }

        info!("register_cache setter exiting");

        Ok(())
    }
}
