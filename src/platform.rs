use crossbeam_channel::{bounded, Receiver, Sender};

pub(crate) struct PlatformContext {
    pub keyboard: Sender<[u8; 16]>,
    pub single_key: Sender<char>,
    pub sound: Receiver<bool>,
    pub display: Receiver<[[u8; 32]; 64]>,
}

pub(crate) struct CpuContext {
    pub keyboard: Receiver<[u8; 16]>,
    pub single_key: Receiver<char>,
    pub sound: Sender<bool>,
    pub display: Sender<[[u8; 32]; 64]>,
}

pub(crate) fn create_contexts() -> (PlatformContext, CpuContext) {
    let (ks, kr) = bounded::<[u8; 16]>(1);
    let (sks, skr) = bounded::<char>(0);
    let (ss, sr) = bounded::<bool>(1);
    let (ds, dr) = bounded::<[[u8; 32]; 64]>(2);

    return (
        PlatformContext {
            keyboard: ks,
            single_key: sks,
            sound: sr,
            display: dr,
        },
        CpuContext {
            keyboard: kr,
            single_key: skr,
            sound: ss,
            display: ds,
        },
    );
}

pub(crate) trait Platform {
    fn start(&mut self, context: &PlatformContext);
    fn update(&mut self, context: &PlatformContext);
    fn render(&mut self, context: &PlatformContext);
}
