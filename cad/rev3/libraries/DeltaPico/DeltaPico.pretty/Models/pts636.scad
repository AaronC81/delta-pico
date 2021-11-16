body_width = 6;
body_height = 3.5;
body_depth = 3.3;

button_width = 3;
button_height = 1.5;
button_depth = 0.5;

union() {
    // Body
    cube([body_width, body_height, body_depth]);
    
    // Button
    translate([(body_width - button_width) / 2, (body_height - button_height) / 2, body_depth])
    cube([button_width, button_height, button_depth]);
}
