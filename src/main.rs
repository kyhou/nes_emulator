use macroquad::prelude::*;
use std::collections::BTreeMap;
mod bus;
mod mapper;
mod mapper_000;
use bus::Bus;
mod cpu;
use cpu::Cpu;
mod ppu;
use ppu::{Debug, Ppu};
mod cartridge;
use cartridge::Cartridge;

fn window_conf() -> Conf {
    Conf {
        window_title: "NES_Emulator".to_owned(),
        window_width: 1280,
        window_height: 720,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Load Program (assembled at https://www.masswerk.at/6502/assembler.html)
    /*
        *=$8000
        LDX #10
        STX $0000
        LDX #3
        STX $0001
        LDY $0000
        LDA #0
        CLC
        loop
        ADC $0001
        DEY
        BNE loop
        STA $0002
        NOP
        NOP
        NOP
    */

    let mut ppu = Ppu::new();
    let mut bus = Bus::new();
    let mut cpu = Cpu::new();
    let mut cart = Cartridge::new("nestest.nes");
    let map_asm: BTreeMap<u16, String>;
    let mut emulation_run: bool = false;
    let mut selected_pallete: u8 = 0x00;

    // let program = Vec::from(hex!(
    //     "A2 0A 8E 00 00 A2 03 8E 01 00 AC 00 00 A9 00 18 6D 01 00 88 D0 FA 8D 02 00 EA EA EA"
    // ));
    // cpu.load_program(&mut bus, 0x8000, program, 0x00, 0x80);

    if !cart.image_valid {
        panic!("error loading cartridge image");
    }

    // map_asm = cpu.disassemble(0x0000, 0xFFFF, &mut bus, &mut ppu, &mut cart);

    bus.reset(&mut cpu, &mut ppu, &mut cart);

    let mut fps_timer = 0_f32;
    let mut fps: i32 = 0;
    let mut show_name_tbl: bool = false;

    let target_fps = 60;
    let mut last_frame_time = get_time();

    let main_texture: Texture2D = Texture2D::from_image(&Image::gen_image_color(256, 240, BLACK));
    let image_0_texture: Texture2D = Texture2D::from_image(&Image::gen_image_color(128, 128, BLACK));
    let image_1_texture: Texture2D = Texture2D::from_image(&Image::gen_image_color(128, 128, BLACK));

    loop {
        let current_time = get_time();
        let delta_time = current_time - last_frame_time;
        let target_delta_time = 1.0 / target_fps as f64;

        if delta_time < target_delta_time {
            continue;
        }

        last_frame_time = current_time;

        clear_background(DARKBLUE);

        fps_timer += get_frame_time();

        if fps_timer > 0.2 {
            fps = macroquad::time::get_fps();
            fps_timer = 0f32;
        }

        draw_text(
            &format!("{} {}", &fps, " FPS")[..],
            1200f32,
            25f32,
            20f32,
            WHITE,
        );

        // if is_key_pressed(KeyCode::Space) {
        //     loop {
        //         cpu.clock(&mut bus, &mut ppu, &mut cartridge);
        //         if cpu.complete() {
        //             break;
        //         }
        //     }
        // }

        // if is_key_pressed(KeyCode::R) {
        //     cpu.reset(&mut bus, &mut ppu, &mut cartridge);
        // }

        // if is_key_pressed(KeyCode::I) {
        //     cpu.irq(&mut bus, &mut ppu, &mut cartridge);
        // }

        // if is_key_pressed(KeyCode::N) {
        //     cpu.nmi(&mut bus, &mut ppu, &mut cartridge);
        // }

        // cpu.draw_ram(&mut bus, &mut ppu, &mut cartridge, 2, 12, 0x0000, 16, 16);
        // cpu.draw_ram(&mut bus, &mut ppu, &mut cartridge, 2, 272, 0x8000, 16, 16);
        // cpu.draw_cpu(550, 12);
        // cpu.draw_code(550, 122, 26, &map_asm);

        // draw_text(
        //     "SPACE = Step Instruction    R = RESET    I = IRQ    N = NMI",
        //     10.0,
        //     550.0,
        //     25.0,
        //     WHITE,
        // );

        bus.controller[0] = 0x00;
        bus.controller[0] |= if is_key_down(KeyCode::Z) { 0x80 } else { 0x00 };
        bus.controller[0] |= if is_key_down(KeyCode::X) { 0x40 } else { 0x00 };
        bus.controller[0] |= if is_key_down(KeyCode::S) { 0x20 } else { 0x00 };
        bus.controller[0] |= if is_key_down(KeyCode::A) { 0x10 } else { 0x00 };
        bus.controller[0] |= if is_key_down(KeyCode::Up) { 0x08 } else { 0x00 };
        bus.controller[0] |= if is_key_down(KeyCode::Down) {
            0x04
        } else {
            0x00
        };
        bus.controller[0] |= if is_key_down(KeyCode::Left) {
            0x02
        } else {
            0x00
        };
        bus.controller[0] |= if is_key_down(KeyCode::Right) {
            0x01
        } else {
            0x00
        };

        if emulation_run {
            while !ppu.frame_complete {
                bus.clock(&mut cpu, &mut ppu, &mut cart);
            }

            ppu.frame_complete = false;
        } else {
            if is_key_pressed(KeyCode::C) {
                while cpu.complete() {
                    bus.clock(&mut cpu, &mut ppu, &mut cart);
                }

                while !cpu.complete() {
                    bus.clock(&mut cpu, &mut ppu, &mut cart);
                }
            }

            if is_key_pressed(KeyCode::F) {
                while ppu.frame_complete {
                    bus.clock(&mut cpu, &mut ppu, &mut cart);
                }

                while !cpu.complete() {
                    bus.clock(&mut cpu, &mut ppu, &mut cart);
                }

                ppu.frame_complete = false;
            }
        }

        if is_key_pressed(KeyCode::R) {
            bus.reset(&mut cpu, &mut ppu, &mut cart)
        }

        if is_key_pressed(KeyCode::Space) {
            emulation_run = !emulation_run;
        }

        if is_key_pressed(KeyCode::P) {
            selected_pallete = selected_pallete.wrapping_add(1) & 0x07;
        }

        // cpu.draw_ram(&mut bus, &mut ppu, &mut cart, 2, 272, 0x8000, 16, 16);
        cpu.draw_cpu(550, 12);
        // cpu.draw_code(&cpu.pc, 550, 122, 26, &map_asm);
        // cpu.draw_ram(&mut bus, &mut ppu, &mut cart, 550, 450, 0x0000, 16, 16);

        for i in 0_usize..24 {
            let oam_reg = ppu.oam[i];

            let mut s = format!("{:2x}", i);
            s.push_str(": (");
            s.push_str(format!("{:3}", oam_reg.x).as_str());
            s.push_str(", ");
            s.push_str(format!("{:3}", oam_reg.y).as_str());
            s.push_str(") ");
            s.push_str("ID: ");
            s.push_str(format!("{:2x}", oam_reg.id).as_str());
            s.push_str(" AT: ");
            s.push_str(format!("{:2x}", oam_reg.attribute).as_str());
            draw_text(s.as_str(), 550.0, (110 + i * 14) as f32, 25.0, WHITE);
        }

        let main_image = ppu.get_screen();

        if is_key_pressed(KeyCode::PrintScreen)
            && (is_key_down(KeyCode::LeftControl) || is_key_down(KeyCode::RightControl))
        {
            main_image.export_png("main_image.png");
        }

        main_texture.update(main_image);
        draw_texture_ex(
            main_texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    (main_image.width * 2) as f32,
                    (main_image.height * 2) as f32,
                )),
                source: None,
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        );

        let swatch_size = 6;
        for p in 0_u8..8 {
            for s in 0_u8..4 {
                // println!("Color: {:?}", ppu.get_colour_from_pallet_ram(&mut cart, &p, &(s as u8)));
                // println!("x: {:?}", (550 + (p as i32 * swatch_size * 5) + (s * swatch_size)) as f32);
                draw_rectangle(
                    (550 + (p as i32 * (swatch_size * 5)) + (s as i32 * swatch_size)) as f32,
                    440.0,
                    swatch_size as f32,
                    swatch_size as f32,
                    ppu.get_colour_from_pallet_ram(&mut cart, p, s),
                )
            }
        }

        draw_rectangle_lines(
            (550 + (selected_pallete as i32 * (swatch_size * 5)) - 1) as f32,
            440.0,
            ((swatch_size * 4) + 1) as f32,
            (swatch_size + 1) as f32,
            2.0,
            YELLOW,
        );

        let image_0 = ppu.get_pattern_table(0, &selected_pallete, &mut cart);
        image_0_texture.update(image_0);
        draw_texture_ex(
            image_0_texture,
            550.0,
            450.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    (image_0.width * 2) as f32,
                    (image_0.height * 2) as f32,
                )),
                source: None,
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        );

        let image_1 = ppu.get_pattern_table(1, &selected_pallete, &mut cart);
        image_1_texture.update(image_1);
        draw_texture_ex(
            image_1_texture,
            820.0,
            450.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(
                    (image_1.width * 2) as f32,
                    (image_1.height * 2) as f32,
                )),
                source: None,
                rotation: 0.0,
                flip_x: false,
                flip_y: false,
                pivot: None,
            },
        );

        next_frame().await
    }
}
