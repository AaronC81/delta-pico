delta_pico_width = 78.2;
delta_pico_height = 167.2;
delta_pico_corner_radius = 4.5;

border_size = 3;

power_switch_offset = 8;
power_switch_length = 10;
jst_offset = 4;
jst_length = 9;
usb_offset = 50;
usb_length = 14;

screws_edge_distance = 6;
bottom_screws_offset = 6;
middle_screws_offset = 76.1;
top_screw_offset = 142.25;

edge_cutout_size = border_size + 1;

base_height = 4;
screw_depth = 6;
peg_height = 5;

module delta_pico() {
    translate([delta_pico_corner_radius, delta_pico_corner_radius])
    minkowski() {
        circle(delta_pico_corner_radius);
        square([
            delta_pico_width - delta_pico_corner_radius * 2,
            delta_pico_height - delta_pico_corner_radius * 2
        ]);
    }
}

module delta_pico_outline() {
    difference() {
        translate([border_size + delta_pico_corner_radius, border_size + delta_pico_corner_radius])
        minkowski() {
            circle(border_size + delta_pico_corner_radius);
            square([
                delta_pico_width - delta_pico_corner_radius * 2,
                delta_pico_height - delta_pico_corner_radius * 2
            ]);
        }

        // Cut out the calculator itself
        translate([border_size, border_size])
        delta_pico();
    }
}

module delta_pico_outline_with_cutouts() {
    difference() {
        delta_pico_outline();
        
        // Cut out the power switch
        translate([0, border_size + delta_pico_height - power_switch_offset - power_switch_length])
        square([edge_cutout_size, power_switch_length]);
        
        // Cut out the JST connector
        translate([border_size + jst_offset, border_size + delta_pico_height - 1])
        square([jst_length, edge_cutout_size]);
        
        // Cut out the USB port
        translate([border_size + usb_offset, border_size + delta_pico_height - 1])
        square([usb_length, edge_cutout_size]);
    }
}

module screw_hole() {
    linear_extrude(screw_depth)
    circle(d=2.7, $fn=15);
}

module screw_peg() {
    linear_extrude(peg_height)
    circle(d=6, $fn=15);
}

module base() {
    difference() {
        union() {
            linear_extrude(base_height)
            delta_pico_outline();
            
            translate([border_size, border_size]) {
                linear_extrude(base_height)
                delta_pico();
                
                translate([6, 6, base_height])
                screw_peg();
                
                translate([delta_pico_width - 6, 6, base_height])
                screw_peg();
                
                translate([6, middle_screws_offset, base_height])
                screw_peg();
                
                translate([delta_pico_width - 6, middle_screws_offset, base_height])
                screw_peg();
                
                translate([6, top_screw_offset, base_height])
                screw_peg();
            }
        }
        
        translate([border_size, border_size]) {
            screw_offset = base_height - screw_depth + peg_height;
            
            translate([6, 6, screw_offset])
            screw_hole();
            
            translate([delta_pico_width - 6, 6, screw_offset])
            screw_hole();
            
            translate([6, middle_screws_offset, screw_offset])
            screw_hole();
                
            translate([delta_pico_width - 6, middle_screws_offset, screw_offset])
            screw_hole();
            
            translate([6, top_screw_offset, screw_offset])
            screw_hole();
        }
    }
}

module logo() {
    translate([3 - 60 / 2, 3 - 50 / 2])
    minkowski() {
        polygon([
            [0, 0],
            [30, 50],
            [60, 0],
            [0, 0.1],
            [60, 0.1],
            [30, 50.1],
            [0, 0.1]
        ]);
        circle(4, $fn=30);
    }
}

difference() {
    base();

    translate([delta_pico_width / 2, delta_pico_height / 2])
    linear_extrude(base_height - 2.5)
    logo();
}

translate([0, 0, base_height])
linear_extrude(7.5)
delta_pico_outline_with_cutouts();
