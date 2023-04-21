#![no_std]
#![no_main]

use core::mem::MaybeUninit;
use defmt_rtt as _;
use dwt_systick_monotonic::{DwtSystick, ExtU32};
use heapless::pool::{Box, Node, Pool};
use panic_probe as _;
use stm32l4xx_hal::pac::USART1;
use stm32l4xx_hal::prelude::*;
use stm32l4xx_hal::serial::{Config, Event, Rx, Serial};
use tp_led_matrix::{matrix::Matrix, Image};

#[rtic::app(device = stm32l4xx_hal::pac, dispatchers = [USART2, USART3])]
mod app {
    use tp_led_matrix::Color;

    use super::*;

    #[monotonic(binds = SysTick, default = true)]
    type MyMonotonic = DwtSystick<80_000_000>;
    type Instant = <MyMonotonic as rtic::Monotonic>::Instant;

    #[shared]
    struct Shared {
        next_image: Option<Box<Image>>,
        pool: Pool<Image>,
        changes: u32,
    }

    #[local]
    struct Local {
        matrix: Matrix,
        usart1_rx: Rx<USART1>,
        current_image: Box<Image>,
        rx_image: Box<Image>,
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        defmt::info!("defmt correctly initialized");

        let mut cp = cx.core;
        let dp = cx.device;

        // Initialize the clocks, hardware and matrix using your existing code
        let mut mono = DwtSystick::new(&mut cp.DCB, cp.DWT, cp.SYST, 80_000_000);

        // Get high-level representations of hardware modules
        let mut rcc = dp.RCC.constrain();
        let mut flash = dp.FLASH.constrain();
        let mut pwr = dp.PWR.constrain(&mut rcc.apb1r1);

        // Setup the clocks at 80MHz using HSI (by default since HSE/MSI are not configured).
        // The flash wait states will be configured accordingly.
        let clocks = rcc.cfgr.sysclk(80.MHz()).freeze(&mut flash.acr, &mut pwr);
        let mut gpioa = dp.GPIOA.split(&mut rcc.ahb2);
        let mut gpiob = dp.GPIOB.split(&mut rcc.ahb2);
        let mut gpioc = dp.GPIOC.split(&mut rcc.ahb2);
        let matrix = Matrix::new(
            gpioa.pa2,
            gpioa.pa3,
            gpioa.pa4,
            gpioa.pa5,
            gpioa.pa6,
            gpioa.pa7,
            gpioa.pa15,
            gpiob.pb0,
            gpiob.pb1,
            gpiob.pb2,
            gpioc.pc3,
            gpioc.pc4,
            gpioc.pc5,
            &mut gpioa.moder,
            &mut gpioa.otyper,
            &mut gpiob.moder,
            &mut gpiob.otyper,
            &mut gpioc.moder,
            &mut gpioc.otyper,
            clocks,
        );

        // the display task gets spawned after init() terminates
        display::spawn(mono.now()).unwrap();
        screensaver::spawn(mono.now()).unwrap();

        // Serial port
        let tx_pin =
            gpiob
                .pb6
                .into_alternate::<7>(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);
        let rx_pin =
            gpiob
                .pb7
                .into_alternate::<7>(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl);

        let mut usart1_config: Config = stm32l4xx_hal::serial::Config::default();
        usart1_config = usart1_config.baudrate(38_400.bps());

        let mut serial = Serial::usart1(
            dp.USART1,
            (tx_pin, rx_pin),
            usart1_config,
            clocks,
            &mut rcc.apb2,
        );

        serial.listen(Event::Rxne);

        let data = serial.split();
        let usart1_rx = data.1;

        // Triple buffering (inside pool)
        let pool: Pool<Image> = Pool::new();
        unsafe {
            static mut MEMORY: MaybeUninit<[Node<Image>; 3]> = MaybeUninit::uninit();
            pool.grow_exact(&mut MEMORY); // static mut access is unsafe
        }

        let current_image = pool.alloc().unwrap().init(Image::default());
        let rx_image = pool.alloc().unwrap().init(Image::default()); // MaybeUninit::init
        let next_image = None;
        let changes = 0;

        // Return the resources and the monotonic timer
        return (
            Shared {
                next_image,
                pool,
                changes,
            },
            Local {
                matrix,
                usart1_rx,
                current_image,
                rx_image,
            },
            init::Monotonics(mono),
        );
    }

    #[idle]
    fn idle(_cx: idle::Context) -> ! {
        loop {}
    }

    #[task(local = [matrix, next_row: usize = 0, current_image], shared = [&pool, next_image], priority = 2)]
    fn display(mut cx: display::Context, at: Instant) {
        // Display line next_line (cx.local.next_line) of
        // the image (cx.local.image) on the matrix (cx.local.matrix).
        // All those are mutable references.
        // cx.local.current_image.lock(|current| {
        // Here you can use image, which is a &mut Image,
        // to display the appropriate row
        cx.local.matrix.send_row(
            *cx.local.next_row,
            cx.local.current_image.row(*cx.local.next_row),
        );

        if *cx.local.next_row as usize == 7 {
            cx.shared.next_image.lock(|next_image| {
                if next_image.is_none() == false {
                    if let Some(mut image) = next_image.take() {
                        core::mem::swap(&mut image, cx.local.current_image.into());
                        cx.shared.pool.free(image);
                    }
                }
            })
        }
        // });

        // Increment next_row up to 7 and wraparound to 0
        *cx.local.next_row = (*cx.local.next_row + 1) % 8;

        // it gets respawned
        let next = at + (1.secs() / 8 / 60);
        display::spawn_at(next, next).unwrap();
    }

    #[task(binds = USART1, local = [usart1_rx, next_pos: usize = 0, rx_image], shared = [next_image, &pool])]
    fn receive_byte(mut cx: receive_byte::Context) {
        let next_pos: &mut usize = cx.local.next_pos;

        if let Ok(b) = cx.local.usart1_rx.read() {
            let error = cx.local.usart1_rx.check_for_error();
            match error {
                Ok(()) => {
                    defmt::info!("Ok");
                }
                Err(_error) => {
                    return;
                }
            }
            if b == 0xff {
                *next_pos = 0;
            } else if *next_pos == usize::MAX {
                // blocking untill arriving of a new image
            } else if let Some(image) = cx.local.rx_image.into() {
                image.as_mut()[*next_pos] = b;
                *next_pos += 1;
            } else {
                defmt::error!("Error while reading incoming data");
            }

            // If the received image is complete, make it available to
            // the display task.
            if *next_pos == 8 * 8 * 3 {
                cx.shared.next_image.lock(|next_image| {
                    if next_image.is_none() != false {
                        if let Some(image) = next_image.take() {
                            cx.shared.pool.free(image);
                        }
                    }
                    // Replace the image content by the new one, for example
                    // by swapping them, and reset next_pos
                    let mut future_image = cx.shared.pool.alloc().unwrap().init(Image::default());
                    core::mem::swap(&mut future_image, &mut cx.local.rx_image);
                    *next_image = Some(future_image);
                    notice_change::spawn().unwrap();
                });
                *next_pos = usize::MAX;
            }
        }
    }

    #[task(shared = [changes], priority = 3)]
    fn notice_change(mut cx: notice_change::Context) {
        cx.shared.changes.lock(|changes| {
            *changes = (*changes + 1) % (2u32.pow(32) - 1);
        })
    }

    #[task(local = [last_changes: u32 = 0, color_index: u8 = 0], shared = [next_image, &pool, changes], priority=2)]
    fn screensaver(mut cx: screensaver::Context, at: Instant) {
        let last_changes: &mut u32 = cx.local.last_changes;
        let color_index: &mut u8 = cx.local.color_index as &mut u8;
        let color_now = if *color_index == 0 {
            Color::RED
        } else if *color_index == 1 {
            Color::GREEN
        } else {
            Color::BLUE
        };

        cx.shared.changes.lock(|changes| {
            cx.shared.next_image.lock(|next_image| {
                if *last_changes == *changes {
                    let gradient = cx
                        .shared
                        .pool
                        .alloc()
                        .unwrap()
                        .init(Image::gradient(color_now));
                    *next_image = Some(gradient);
                    *color_index = (*color_index + 1) % 3;
                } else {
                    if let Some(image) = next_image.take() {
                        cx.shared.pool.free(image);
                    }
                    *last_changes = *changes;
                }
            });
        });

        let next = at + 1.secs();
        screensaver::spawn_at(next, next).unwrap();
    }
}
