#![no_std]
#![no_main]
#![feature(impl_trait_in_assoc_type)]

use cortex_m_rt as _;
use defmt_rtt as _;
use panic_probe as _;

use embassy_executor::Spawner;
use embassy_stm32::{
    bind_interrupts, gpio,
    mode::Async,
    peripherals,
    usart::{self, InterruptHandler, Uart, UartRx},
};
use embassy_time::{Duration, Ticker};

bind_interrupts!(struct Irqs {
    USART2 => InterruptHandler<peripherals::USART2>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) -> ! {
    let mut embassy_config = embassy_stm32::Config::default();
    configure_rcc(&mut embassy_config.rcc);

    let resources = embassy_stm32::init(embassy_config);

    // Create the DMA-based Uart
    let mut usart_config = usart::Config::default();
    usart_config.baudrate = 115_200;
    usart_config.rx_pull = gpio::Pull::Up;

    let usart = Uart::new(
        resources.USART2,
        resources.PD6,
        resources.PD5,
        Irqs,
        resources.GPDMA1_CH0,
        resources.GPDMA1_CH1,
        usart_config,
    )
    .expect("Invalid UART Config!");

    let (mut tx, rx) = usart.split();

    let dummy_data: [u8; _] = [0xCA, 0xFE, 0xBA, 0xBE, 0xDA, 0xDA];
    let mut ticker = Ticker::every(Duration::from_secs(1));

    spawner.must_spawn(read_task(dummy_data.len(), rx));

    loop {
        // defmt::info!("Awaiting Ticker");
        ticker.next().await;

        defmt::info!("Sending data: {:#04X}", dummy_data);
        tx.write(&dummy_data).await.expect("Failure to write data");
    }
}

fn configure_rcc(config: &mut embassy_stm32::rcc::Config) {
    use embassy_stm32::rcc::{
        AHBPrescaler, APBPrescaler, MSIRange, Pll, PllDiv, PllMul, PllPreDiv, PllSource, Sysclk,
        mux::{Usart1sel, Usartsel},
    };

    config.msis = Some(MSIRange::RANGE_48MHZ);
    config.pll1 = Some(Pll {
        source: PllSource::MSIS,
        prediv: PllPreDiv::DIV3,
        mul: PllMul::MUL10,
        divp: None,
        divq: None,
        divr: Some(PllDiv::DIV1),
    });
    config.sys = Sysclk::PLL1_R;

    config.ahb_pre = AHBPrescaler::DIV1;
    config.apb1_pre = APBPrescaler::DIV16;
    config.apb2_pre = APBPrescaler::DIV16;

    config.mux.usart1sel = Usart1sel::PCLK2;
    config.mux.usart2sel = Usartsel::PCLK1;
    config.mux.usart3sel = Usartsel::PCLK1;
}

#[embassy_executor::task]
async fn read_task(data_len: usize, mut rx: UartRx<'static, Async>) -> ! {
    let mut buffer;
    let mut cycle = 1_u8;

    loop {
        defmt::info!("Resetting buffer...");
        buffer = [0_u8; 20];

        let amount_read;

        let change_destination = cfg!(feature = "change-destination");
        let read_until_idle = cfg!(feature = "read-until-idle");

        match (change_destination, read_until_idle) {
            (true, true) => {
                let buffer_ref;

                if cycle.is_multiple_of(4) {
                    defmt::info!("Reading into address 8...");
                    buffer_ref = &mut buffer[8..];
                    cycle = 1_u8;
                } else {
                    defmt::info!("Reading into address 0...");
                    buffer_ref = &mut buffer;
                    cycle += 1;
                }

                amount_read = rx
                    .read_until_idle(buffer_ref)
                    .await
                    .expect("UART Read Failure!");
            }
            (true, false) => {
                let buffer_ref;

                if cycle.is_multiple_of(4) {
                    defmt::info!("Reading into address 8...");
                    buffer_ref = &mut buffer[8..8 + data_len];
                    cycle = 1_u8;
                } else {
                    defmt::info!("Reading into address 0...");
                    buffer_ref = &mut buffer[..data_len];
                    cycle += 1;
                }

                rx.read(buffer_ref).await.expect("UART Read Failure!");
                amount_read = data_len;
            }
            (false, true) => {
                amount_read = rx
                    .read_until_idle(&mut buffer)
                    .await
                    .expect("UART Read Failure!");
            }
            (false, false) => {
                let exact_size_slice = &mut buffer[..data_len];
                rx.read(exact_size_slice).await.expect("UART Read Failure");
                amount_read = data_len;
            }
        }

        defmt::info!("Read {} bytes: {:#04X}", amount_read, &buffer);
    }
}
