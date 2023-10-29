use crate::prelude::*;

// this just needs to be bigger than the max register we'll see
const REGISTER_COUNT: usize = 256;

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum ChannelData {
    RegisterData(u16, u16),
    Shutdown,
}

pub struct RegisterCache {
    config: ConfigWrapper,
    channels: Channels,
    register_data: Rc<RefCell<[u16; REGISTER_COUNT]>>,
}

impl RegisterCache {
    pub fn new(config: ConfigWrapper, channels: Channels) -> Self {
        let register_data = Rc::new(RefCell::new([0; REGISTER_COUNT]));

        Self {
            config,
            channels,
            register_data,
        }
    }

    pub async fn start(&self) -> Result<()> {
        futures::try_join!(self.receiver())?;

        Ok(())
    }

    async fn receiver(&self) -> Result<()> {
        use ChannelData::*;

        let mut receiver = self.channels.to_register_cache.subscribe();

        loop {
            match receiver.recv().await? {
                RegisterData(register, value) => {
                    if register < REGISTER_COUNT as u16 {
                        self.save(register, value);
                    } else {
                        warn!(
                            "cannot cache register {}, increase REGISTER_COUNT!",
                            register
                        );
                    }
                }
                Shutdown => break,
            }
        }

        Ok(())
    }

    fn save(&self, register: u16, value: u16) {
        let mut register_data = self.register_data.borrow_mut();
        register_data[register as usize] = value;
    }
}
