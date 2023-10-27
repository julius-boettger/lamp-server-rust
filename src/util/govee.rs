type RGBColor = (u8, u8, u8);

enum SetStatus {
    Color(RGBColor),
    Brightness(u8),
    Power(bool)
}

fn set_status(status: SetStatus) {
    
}