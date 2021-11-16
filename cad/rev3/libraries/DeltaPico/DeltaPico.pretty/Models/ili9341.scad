display_board_width = 50;
display_board_height = 81;
display_board_depth = 1.57;

pin_spacing = 2.54;
pin_radius = 0.7; // TODO: estimate

display_board_main_pins_offset_y = 1.5; // roughly
display_board_main_pins_count = 14;

display_panel_border_height = 69; // nice
display_panel_border_offset_y = 6;
display_panel_border_depth = 3; // TODO: eyeballed

display_panel_width = 46;
display_panel_height = 65;
display_panel_offset_y = 2.5;

sd_slot_height = 26.2;
sd_slot_offset_y = 30;
sd_slot_width = 20; // TODO: eyeballed
sd_slot_depth = 3; // TODO: eyeballed

union() {
    
// Display board
difference() {
    color("red")
    cube([display_board_width, display_board_height, display_board_depth]);
    
    translate([
        (display_board_width - display_board_main_pins_count * pin_spacing) / 2,
        display_board_main_pins_offset_y,
        0
    ])
    for (i = [ 0 : display_board_main_pins_count - 1 ]) {
        translate([i * pin_spacing, 0, 0])
        cylinder(display_board_depth, r=pin_radius, $fn=20);
    }
}

// Display panel and border
translate([0, display_panel_border_offset_y, display_board_depth]) {
    // Border
    color("white")
    cube([display_board_width, display_panel_border_height, display_panel_border_depth]);

    // Panel
    color("black")
    translate([(display_board_width - display_panel_width) / 2, display_panel_offset_y, display_panel_border_depth])
    cube([display_panel_width, display_panel_height, 0.1]);
}

// SD slot
color("grey")
translate([0, sd_slot_offset_y, -sd_slot_depth])
cube([sd_slot_width, sd_slot_height, sd_slot_depth]);

}