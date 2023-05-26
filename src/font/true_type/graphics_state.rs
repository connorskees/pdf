#[derive(Debug)]
pub struct TrueTypeGraphicsState {
    pub auto_flip: bool,
    pub control_value_cut_in: (),
    pub delta_base: (),
    pub delta_shift: (),
    pub dual_projection_vector: (),
    pub freedom_vector: (),
    pub instruct_control: (),
    pub loop_amt: (),
    pub minimum_distance: (),
    pub projection_vector: (),
    pub round_state: (),
    pub rp0: (),
    pub rp1: (),
    pub rp2: (),
    pub scan_control: (),
    pub single_width_cut_in: (),
    pub single_width_value: (),
    pub zp0: (),
    pub zp1: (),
    pub zp2: (),
}

impl Default for TrueTypeGraphicsState {
    fn default() -> Self {
        Self {
            auto_flip: true,
            control_value_cut_in: (),
            delta_base: (),
            delta_shift: (),
            dual_projection_vector: (),
            freedom_vector: (),
            instruct_control: (),
            loop_amt: (),
            minimum_distance: (),
            projection_vector: (),
            round_state: (),
            rp0: (),
            rp1: (),
            rp2: (),
            scan_control: (),
            single_width_cut_in: (),
            single_width_value: (),
            zp0: (),
            zp1: (),
            zp2: (),
        }
    }
}
