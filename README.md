# Embassy STM32 U575Z UART DMA Error

This repository provides a minimum reproducible example of an issue I
noticed with Embassy's UART DMA implementation (maybe just the DMA
implementation) on the STM32U575. This issue causes DMA-based reads from
the UART peripheral to enter a bad state if the DMA request is canceled,
for example when using the `read_until_idle` method.

In this state, if the address of the DMA request is not changed, every
other read will contain zero bytes equivalent of the incoming message
length, essentially doubling the read size. For example, the reads would
look somthing like this:

```
    Read # 1: [0xCA, 0xFE]
    Read # 2: [0x00, 0x00, 0xCA, 0xFE]
```

Despite the incoming information in both cases being exactly the same.
The `read_until_idle` method also reports that 4 bytes have been read,
despite the fact that only 2 bytes have been transmitted over the
interface (this fact was also verified with a logic analyzer).

When the address of the read is changed, the behavior becomes even more
unpredictable, and while I've been analyzing different cycle counts I
still haven't been able to find a pattern.
