trait OldAudioJack {
    fn play_sound(&self);
}

trait UsbCAudio {
    fn play_digital_sound(&self);
}

struct VintageHeadphones;

impl OldAudioJack for VintageHeadphones {
    fn play_sound(&self) {
        println!("Playing sound through the vintage headphones!");
    }
}

struct ModernAudioDevice;

impl UsbCAudio for ModernAudioDevice {
    fn play_digital_sound(&self) {
        println!("Playing digital sound through USB-C!");
    }
}

struct AudioAdapter {
    old_headphones: VintageHeadphones,
}

impl UsbCAudio for AudioAdapter {
    fn play_digital_sound(&self) {
        println!("Converting digital sound to analog...");
        self.old_headphones.play_sound();
    }
}



#[test]
fn test(){
    let old_headphones = VintageHeadphones;
    let adapter = AudioAdapter { old_headphones };

    let modern_device = ModernAudioDevice;
    
    // 现在可以通过 modern_device 播放音频，但实际上是通过老式耳机播放
    adapter.play_digital_sound();
}