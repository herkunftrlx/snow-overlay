use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::ptr::{null, null_mut};
use std::thread::sleep;
use std::time::{Duration, Instant};

use windows_sys::Win32::Foundation::*;
use windows_sys::Win32::Graphics::Gdi::*;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
use windows_sys::Win32::UI::WindowsAndMessaging::*;

#[link(name = "winmm")]
unsafe extern "system" {
    fn timeBeginPeriod(uPeriod: u32) -> u32;
    fn timeEndPeriod(uPeriod: u32) -> u32;
}

const HOTKEY_TOGGLE: i32 = 1;
const HOTKEY_QUIT: i32 = 2;
const HOTKEY_FPS: i32 = 3;
const HOTKEY_MONITOR: i32 = 4;

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(Some(0)).collect()
}

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Self(if seed == 0 { 0x853c49e6748fea9b } else { seed })
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u64() & 0xFFFFFF) as f32 / 16777216.0
    }

    fn gen_range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next_f32() * (max - min)
    }
}

#[derive(Clone, Copy)]
struct Snowflake {
    base_x: f32,
    y: f32,
    vy_pps: f32,
    radius: i32,
    sway_phase: f32,
    sway_speed_rad: f32,
    sway_amplitude: f32,
    brush_idx: usize,
}

impl Snowflake {
    fn random(width: f32, height: f32, rng: &mut Rng, initial: bool) -> Self {
        let depth = rng.next_f32();

        let radius = if depth > 0.75 {
            4
        } else if depth > 0.35 {
            3
        } else {
            2
        };

        let vy_pps = 65.0 + depth * 115.0;
        let sway_amplitude = 12.0 + rng.next_f32() * 26.0;
        let sway_speed_rad = 1.2 + rng.next_f32() * 1.8;
        let brush_idx = if depth > 0.7 { 0 } else if depth > 0.35 { 1 } else { 2 };

        let y = if initial {
            rng.gen_range(0.0, height)
        } else {
            -15.0 - rng.gen_range(0.0, 40.0)
        };

        Self {
            base_x: rng.gen_range(0.0, width),
            y,
            vy_pps,
            radius,
            sway_phase: rng.gen_range(0.0, std::f32::consts::TAU),
            sway_speed_rad,
            sway_amplitude,
            brush_idx,
        }
    }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
}

fn main() {
    println!();
    println!("  ========================================================");
    println!("     * * *  DENİZ'S 144Hz SNOW OVERLAY + FPS SAYACI  * * *");
    println!("  ========================================================");
    println!("  [OK] Ana Ekran 144Hz Kilidi (DWM 60Hz mouse bug'ı çözüldü!)");
    println!("  [OK] Gerçek Zamanlı Ekranda FPS Sayacı");
    println!("  [OK] 1ms High-Precision Timer (timeBeginPeriod)");
    println!();
    println!("  KISAYOLLAR:");
    println!("    [Ctrl + Alt + S] : Karı Durdur / Başlat");
    println!("    [Ctrl + Alt + F] : FPS Sayacını Aç / Kapat");
    println!("    [Ctrl + Alt + M] : Ana Ekran / Çift Ekran Değiştir");
    println!("    [Ctrl + Alt + Q] : Çıkış");
    println!("  ========================================================");

    unsafe {
        timeBeginPeriod(1);

        let h_instance = GetModuleHandleW(null());
        let class_name = to_wide("Deniz144HzSnowFpsClass");

        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: 0,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: null_mut(),
            lpszMenuName: null(),
            lpszClassName: class_name.as_ptr(),
            hIconSm: null_mut(),
        };

        RegisterClassExW(&wc);

        // Default to Primary Monitor (0, 0 to 1920x1080) so DWM doesn't downclock to 60Hz!
        let mut dual_mode = false;
        let prim_w = GetSystemMetrics(SM_CXSCREEN);
        let prim_h = GetSystemMetrics(SM_CYSCREEN);

        let virt_x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let virt_y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let virt_w = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let virt_h = GetSystemMetrics(SM_CYVIRTUALSCREEN);

        let mut cur_x = 0;
        let mut cur_y = 0;
        let mut cur_w = prim_w;
        let mut cur_h = prim_h;

        let ex_style = WS_EX_TOPMOST
            | WS_EX_LAYERED
            | WS_EX_TRANSPARENT
            | WS_EX_TOOLWINDOW
            | WS_EX_NOACTIVATE;

        let hwnd = CreateWindowExW(
            ex_style,
            class_name.as_ptr(),
            to_wide("SnowOverlay").as_ptr(),
            WS_POPUP | WS_VISIBLE,
            cur_x,
            cur_y,
            cur_w,
            cur_h,
            null_mut(),
            null_mut(),
            h_instance,
            null_mut(),
        );

        if hwnd.is_null() {
            eprintln!("Pencere oluşturulamadı: {}", GetLastError());
            timeEndPeriod(1);
            return;
        }

        SetLayeredWindowAttributes(hwnd, 0x00000000, 0, LWA_COLORKEY);
        ShowWindow(hwnd, SW_SHOW);
        SetWindowPos(hwnd, HWND_TOPMOST, cur_x, cur_y, cur_w, cur_h, SWP_SHOWWINDOW);

        RegisterHotKey(hwnd, HOTKEY_TOGGLE, (MOD_CONTROL | MOD_ALT) as u32, b'S' as u32);
        RegisterHotKey(hwnd, HOTKEY_QUIT, (MOD_CONTROL | MOD_ALT) as u32, b'Q' as u32);
        RegisterHotKey(hwnd, HOTKEY_FPS, (MOD_CONTROL | MOD_ALT) as u32, b'F' as u32);
        RegisterHotKey(hwnd, HOTKEY_MONITOR, (MOD_CONTROL | MOD_ALT) as u32, b'M' as u32);

        let hdc_win = GetDC(hwnd);
        let hdc_mem = CreateCompatibleDC(hdc_win);

        // Allocate bitmap for maximum virtual screen size so switching modes doesn't reallocate
        let max_w = if virt_w > prim_w { virt_w } else { prim_w };
        let max_h = if virt_h > prim_h { virt_h } else { prim_h };
        let hbm_mem = CreateCompatibleBitmap(hdc_win, max_w, max_h);
        let old_hbm = SelectObject(hdc_mem, hbm_mem);

        let null_pen = GetStockObject(NULL_PEN);
        let old_pen = SelectObject(hdc_mem, null_pen);

        let brushes = [
            CreateSolidBrush(0x00FFFFFF), // Pure white
            CreateSolidBrush(0x00FFF5EA), // Cool white
            CreateSolidBrush(0x00FFE2D0), // Ice white
        ];

        // Text setup for FPS counter
        SetBkMode(hdc_mem, TRANSPARENT as i32);
        SetTextColor(hdc_mem, 0x0000FF7F); // Spring green: BGR = 0x7F, 0xFF, 0x00

        let mut rng = Rng::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64,
        );

        let flake_count = 120;
        let mut flakes: Vec<Snowflake> = (0..flake_count)
            .map(|_| Snowflake::random(cur_w as f32, cur_h as f32, &mut rng, true))
            .collect();

        let mut running = true;
        let mut active = true;
        let mut show_fps = true;

        let target_frame_time = Duration::from_nanos(6_944_444); // 144 FPS
        let mut last_instant = Instant::now();

        // FPS measurement variables
        let mut frame_count: u32 = 0;
        let mut fps_timer = Instant::now();
        let mut displayed_fps: u32 = 144;
        let mut fps_wide_str = to_wide("144 FPS ❄");

        println!("  Kar yağışı 144Hz ana ekranda başlatıldı! ❄️");

        while running {
            let frame_start = Instant::now();
            let dt = (frame_start - last_instant).as_secs_f32().min(0.05);
            last_instant = frame_start;

            frame_count += 1;
            let fps_elapsed = fps_timer.elapsed();
            if fps_elapsed >= Duration::from_millis(500) {
                displayed_fps = (frame_count as f32 / fps_elapsed.as_secs_f32()).round() as u32;
                frame_count = 0;
                fps_timer = Instant::now();
                fps_wide_str = to_wide(&format!("{} FPS ❄", displayed_fps));
            }

            let mut msg: MSG = std::mem::zeroed();
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    running = false;
                    break;
                }
                if msg.message == WM_HOTKEY {
                    match msg.wParam as i32 {
                        HOTKEY_TOGGLE => {
                            active = !active;
                            if !active {
                                PatBlt(hdc_mem, 0, 0, cur_w, cur_h, BLACKNESS);
                                BitBlt(hdc_win, 0, 0, cur_w, cur_h, hdc_mem, 0, 0, SRCCOPY);
                                println!("  [INFO] Duraklatıldı.");
                            } else {
                                println!("  [INFO] Devam ediyor ❄️");
                            }
                        }
                        HOTKEY_QUIT => {
                            running = false;
                            break;
                        }
                        HOTKEY_FPS => {
                            show_fps = !show_fps;
                            println!("  [INFO] FPS sayacı: {}", if show_fps { "Açık" } else { "Kapalı" });
                        }
                        HOTKEY_MONITOR => {
                            dual_mode = !dual_mode;
                            if dual_mode {
                                cur_x = virt_x;
                                cur_y = virt_y;
                                cur_w = virt_w;
                                cur_h = virt_h;
                                println!("  [INFO] Mod: Çift Ekran ({}x{})", cur_w, cur_h);
                            } else {
                                cur_x = 0;
                                cur_y = 0;
                                cur_w = prim_w;
                                cur_h = prim_h;
                                println!("  [INFO] Mod: 144Hz Ana Ekran ({}x{})", cur_w, cur_h);
                            }
                            SetWindowPos(hwnd, HWND_TOPMOST, cur_x, cur_y, cur_w, cur_h, SWP_SHOWWINDOW);
                            // Respawn flakes for new dimensions
                            flakes = (0..flake_count)
                                .map(|_| Snowflake::random(cur_w as f32, cur_h as f32, &mut rng, true))
                                .collect();
                        }
                        _ => {}
                    }
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            if !running {
                break;
            }

            if active {
                PatBlt(hdc_mem, 0, 0, cur_w, cur_h, BLACKNESS);

                for flake in flakes.iter_mut() {
                    flake.y += flake.vy_pps * dt;
                    flake.sway_phase += flake.sway_speed_rad * dt;

                    let cur_flake_x = flake.base_x + flake.sway_phase.sin() * flake.sway_amplitude;

                    if flake.y - flake.radius as f32 > cur_h as f32 {
                        *flake = Snowflake::random(cur_w as f32, cur_h as f32, &mut rng, false);
                        continue;
                    }

                    let r = flake.radius;
                    let cx = cur_flake_x.round() as i32;
                    let cy = flake.y.round() as i32;

                    SelectObject(hdc_mem, brushes[flake.brush_idx]);
                    Ellipse(hdc_mem, cx - r, cy - r, cx + r, cy + r);
                }

                // Render On-Screen FPS Counter at top-right
                if show_fps {
                    let fps_x = cur_w - 95;
                    let fps_y = 12;
                    TextOutW(
                        hdc_mem,
                        fps_x,
                        fps_y,
                        fps_wide_str.as_ptr(),
                        (fps_wide_str.len() - 1) as i32,
                    );
                }

                BitBlt(hdc_win, 0, 0, cur_w, cur_h, hdc_mem, 0, 0, SRCCOPY);
            }

            // Hybrid sleep for rock-solid 144.0 FPS frame pacing
            let target_time = frame_start + target_frame_time;
            let now = Instant::now();
            if target_time > now {
                let remaining = target_time - now;
                if remaining > Duration::from_millis(2) {
                    sleep(remaining - Duration::from_millis(1));
                }
                while Instant::now() < target_time {
                    std::hint::spin_loop();
                }
            } else if !active {
                sleep(Duration::from_millis(30));
            }
        }

        UnregisterHotKey(hwnd, HOTKEY_TOGGLE);
        UnregisterHotKey(hwnd, HOTKEY_QUIT);
        UnregisterHotKey(hwnd, HOTKEY_FPS);
        UnregisterHotKey(hwnd, HOTKEY_MONITOR);
        for &b in &brushes {
            DeleteObject(b);
        }
        SelectObject(hdc_mem, old_pen);
        SelectObject(hdc_mem, old_hbm);
        DeleteObject(hbm_mem);
        DeleteDC(hdc_mem);
        ReleaseDC(hwnd, hdc_win);
        DestroyWindow(hwnd);
        UnregisterClassW(class_name.as_ptr(), h_instance);
        timeEndPeriod(1);
    }

    println!("  [OK] Program kapatıldı.");
}
