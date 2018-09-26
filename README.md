Gridiron
====================

This library is a work in progress. To use it, you can either use one of the provided finite fields, or you can call the macro to create your own. The two that are included are:

* `fp_480::Fp480`
* `fp_256::Fp256`

These were created like so:

    // p = 65000549695646603732796438742359905742825358107623003571877145026864184071783
    fp!(
        fp_256,   // Name of mod
        Fp256,    // Name of class
        256,      // Number of bits for prime
        4,        // Number of limbs (ceil(bits/64))
        [
            1755467536201717351,  // prime number in limbs, least sig first
            17175472035685840286, // get this from sage with p.digits(2^64)
            12281294985516866593,
            10355184993929758713
        ],
        [
            // barrett reduction for reducing values up to twice
            // the number of prime bits (double limbs):
            // floor(2^(64*numlimbs*2)/p)
            4057416362780367814,
            12897237271039966353,
            2174143271902072370,
            14414317039193118239,
            1
        ]
    );


To use it, you'll need to import headers for the math operations you want. So, for example:

    use std::ops::Add;
    let one = fp_256::Fp256::one();
    let two = one + one;

This is a work in progress and we hope to make it more performant and constant time.