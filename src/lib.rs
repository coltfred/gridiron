extern crate num_traits;
#[cfg(test)]
extern crate rand;
#[cfg(test)]
#[macro_use]
extern crate proptest;

#[macro_use]
pub mod digits {
    pub mod signed;
    pub mod unsigned;
    #[macro_use]
    pub mod ff;
    pub mod util;
}

const BITSPERBYTE: usize = 8;
const U64BYTES: usize = 8;

// p = 3121577065842246806003085452055281276803074876175537384188619957989004527066410274868798956582915008874704066849018213144375771284425395508176023
//   = 0xfffc6664 0e249d9ec75ad529 0b81a85d415797b9 31258da0d78b58a2 1c435cddb02e0add 635a037371d1e9a4 0a5ec1d6ed637bd3 695530683ee96497
fp!(
    fp_480, // Name of mod
    Fp480,  // Name of class
    480,    // Number of bits for prime
    8,      // Number of limbs (ceil(bits/64))
    [
        // prime number in limbs, least sig first
        // get this from sage with p.digits(2^64)
        0x695530683ee96497,
        0x0a5ec1d6ed637bd3,
        0x635a037371d1e9a4,
        0x1c435cddb02e0add,
        0x31258da0d78b58a2,
        0x0b81a85d415797b9,
        0x0e249d9ec75ad529,
        0xfffc6664
    ],
    // barrett reduction for reducing values up to twice
    // the number of prime bits (double limbs):
    // floor(2^(64*numlimbs*2)/p)
    [
        12949131531391198536,
        16219634423243107790,
        3078644475342494540,
        14339002018868860281,
        10620351872007386094,
        16410052731186111519,
        4427379449636958391,
        16707647147458534671,
        4295203240,
    ]
);

// p = 65000549695646603732796438742359905742825358107623003571877145026864184071783
fp!(
    fp_256, // Name of mod
    Fp256,  // Name of class
    256,    // Number of bits for prime
    4,      // Number of limbs (ceil(bits/64))
    [
        1755467536201717351,  // prime number in limbs, least sig first
        17175472035685840286, // get this from sage with p.digits(2^64)
        12281294985516866593,
        10355184993929758713
    ],
    // barrett reduction for reducing values up to twice
    // the number of prime bits (double limbs):
    // floor(2^(64*numlimbs*2)/p)
    [
        4057416362780367814,
        12897237271039966353,
        2174143271902072370,
        14414317039193118239,
        1
    ]
);

impl From<[u8; 64]> for fp_256::Fp256 {
    fn from(src: [u8; 64]) -> Self {
        // our input is the exact length we need for our
        // optimized barrett reduction
        let limbs = eight_limbs_from_sixtyfour_bytes(src);
        fp_256::Fp256::new(fp_256::reduce_barrett(&limbs))
    }
}

impl From<[u8; 64]> for fp_480::Fp480 {
    fn from(src: [u8; 64]) -> Self {
        fp_480::Fp480::new(eight_limbs_from_sixtyfour_bytes(src)).normalize(0)
    }
}

fn eight_limbs_from_sixtyfour_bytes(bytes: [u8; 64]) -> [u64; 8] {
    let mut limbs = [0u64; 8];
    for (i, limb) in limbs.iter_mut().enumerate() {
        for j in (0..U64BYTES).rev() {
            let idx = i * U64BYTES + j;
            *limb <<= BITSPERBYTE;
            *limb |= bytes[64 - idx - 1] as u64;
        }
    }
    limbs
}

#[cfg(test)]
mod lib {
    use super::*;
    use num_traits::{Inv, One, Pow, Zero};
    use std::ops::{Div, Mul};

    #[test]
    fn normalize() {
        // pplusone = fp_480::PRIME + 1
        let pplusone = fp_480::Fp480::new([
            7590025971293054104,
            747247717039963091,
            7159038352024529316,
            2036573563714931421,
            3541392403947280546,
            829128924894566329,
            1019112720967587113,
            4294731364,
        ]);
        assert_eq!(pplusone.normalize(0), fp_480::Fp480::one());

        let ptimestwo = fp_480::Fp480::new([
            15180051942586108206,
            1494495434079926182,
            14318076704049058632,
            4073147127429862842,
            7082784807894561092,
            1658257849789132658,
            2038225441935174226,
            8589462728,
        ]);
        assert_eq!(ptimestwo.normalize(0), fp_480::Fp480::zero());

        let ptimesthreeplusone = fp_480::Fp480::new([
            4323333840169610694,
            2241743151119889274,
            3030370982364036332,
            6109720691144794264,
            10624177211841841638,
            2487386774683698987,
            3057338162902761339,
            12884194092,
        ]);
        assert_eq!(ptimesthreeplusone.normalize(0), fp_480::Fp480::one());

        // so we should never have a number
        // greater than 2p, which means all F's
        // would be invalid for fp480
        let max = fp_480::Fp480::new([
            0xFFFFFFFFFFFFFFFFu64,
            0xFFFFFFFFFFFFFFFFu64,
            0xFFFFFFFFFFFFFFFFu64,
            0xFFFFFFFFFFFFFFFFu64,
            0xFFFFFFFFFFFFFFFFu64,
            0xFFFFFFFFFFFFFFFFu64,
            0xFFFFFFFFFFFFFFFFu64,
            0x00000000FFFFFFFFu64,
        ]);
        // as determined by sage
        let expected = fp_480::Fp480::new([
            10856718102416497512,
            17699496356669588524,
            11287705721685022299,
            16410170509994620194,
            14905351669762271069,
            17617615148814985286,
            17427631352741964502,
            235931,
        ]);
        assert_eq!(max.normalize(0), expected);
    }

    #[test]
    fn is_normalized() {
        let big = fp_480::Fp480::new([0xffffffffu64; fp_480::NUMLIMBS]);
        let p = fp_480::Fp480::new(fp_480::PRIME);
        let one = fp_480::Fp480::one();
        assert!(p + one < p);
        assert!(one - p < p);
        assert!(big * big < p);
        assert!(big.pow(3) < p);
    }

    #[test]
    fn hex_dec_print() {
        let p = fp_480::Fp480::new(fp_480::PRIME);
        assert_eq!(p.to_str_decimal().as_str(),  "3121577065842246806003085452055281276803074876175537384188619957989004527066410274868798956582915008874704066849018213144375771284425395508176023");
        println!(
            "Array: {:?}",
            p.to_bytes_array()
                .iter()
                .map(|x| format!("{:02x}", x))
                .collect::<Vec<String>>()
        );
        assert_eq!(p.to_str_hex().as_str(),  "fffc66640e249d9ec75ad5290b81a85d415797b931258da0d78b58a21c435cddb02e0add635a037371d1e9a40a5ec1d6ed637bd3695530683ee96497");

        let p = fp_256::Fp256::new(fp_256::PRIME);
        assert_eq!(
            p.to_str_decimal().as_str(),
            "65000549695646603732796438742359905742825358107623003571877145026864184071783"
        );
        assert_eq!(
            p.to_str_hex().as_str(),
            "8fb501e34aa387f9aa6fecb86184dc21ee5b88d120b5b59e185cac6c5e089667"
        );

        let mut bytes = [0u8; 32];
        bytes[0] = 255;
        // actual result here is mod PRIME
        assert_eq!(
            fp_256::Fp256::from(bytes).to_str_decimal().as_str(),
            "50339226693086325302401222106137814970392790680417402014301307793518034905497"
        );
    }

    #[test]
    fn to_bytes() {
        let x = fp_256::Fp256::new([0, 0xeeff, 0, 0]);
        let mut expected_bytes = [0u8; 32];
        expected_bytes[22] = 0xeeu8;
        expected_bytes[23] = 0xffu8;
        assert_eq!(x.to_bytes_array(), expected_bytes);

        let x = fp_256::Fp256::new([0, 0, 0, 0xff00000000000000]);
        let mut expected_bytes = [0u8; 32];
        expected_bytes[0] = 0xffu8;
        assert_eq!(x.to_bytes_array(), expected_bytes);
    }

    #[test]
    fn dec_print() {
        let p = fp_480::Fp480::new(fp_480::PRIME);
        assert_eq!(p.to_str_decimal(),  "3121577065842246806003085452055281276803074876175537384188619957989004527066410274868798956582915008874704066849018213144375771284425395508176023");
    }

    #[test]
    fn zero1() {
        let a = fp_480::Fp480::new([1u64; fp_480::NUMLIMBS]);
        assert_eq!(a - a, fp_480::Fp480::zero());
        assert_eq!(a + fp_480::Fp480::zero(), a);
        assert_eq!(a * fp_480::Fp480::zero(), fp_480::Fp480::zero());
    }

    #[test]
    fn mul_precalc() {
        // a = 726838724295606890588725814084399013568056387823382584169080752246511603091378769339537499843825263836454776755850775885168074347773953
        //   = 0x10000000000000001000000000000000100000000000000010000000000000001000000000000000100000000000000010000000000000001
        let a = fp_480::Fp480::new([1u64; fp_480::NUMLIMBS]);
        // a * a % fp_480::PRIME =
        // 0x1374ae453d22515b4d209657a3dbfee9f3bb8cf03f8e24055b70d6ed36bc5e0c7bd2093503dd799794b20d746882519503f026ec8a608fc144f2ec14
        let expected = fp_480::Fp480::new([
            0x8a608fc144f2ec14, // least sig
            0x6882519503f026ec,
            0x03dd799794b20d74,
            0x36bc5e0c7bd20935,
            0x3f8e24055b70d6ed,
            0xa3dbfee9f3bb8cf0,
            0x3d22515b4d209657,
            0x1374ae45, // most sig
        ]);
        assert_eq!(a * a, expected);
    }

    #[test]
    fn static_number_tests() {
        // static values from sage
        // let a = fp_480::Fp480::new_from_string("1849730734868681485455534012483517011951239946690093506001286715509796248056543867438342930397280622761994677963488469871518585836942949575474581", 10).unwrap();
        let a = fp_480::Fp480::new([
            12373281137873304981,
            3364574066891759603,
            18249495978001488097,
            7121407741097929457,
            17616622123341604582,
            999548323268666092,
            102536279974639920,
            2544898439,
        ]);
        // let b = fp_480::Fp480::new_from_string("1427839274828296696112450624937180081894366106872726190963907037699726644896926911406125127099508401951977447182118568992280201726037533400584114", 10).unwrap();
        let b = fp_480::Fp480::new([
            13786755668808912818,
            10295850694267584116,
            6281259092835536344,
            15649415232981746878,
            8768243465836405520,
            8938625549723856276,
            4465923072383974360,
            1964451297,
        ]);
        assert_eq!(
            fp_480::Fp480::zero() - fp_480::Fp480::one(),
            fp_480::Fp480::new(fp_480::PRIME) - fp_480::Fp480::one()
        );
        assert_eq!(
            a + b,
            fp_480::Fp480::new([
                123266761679612080,
                12913177044119380629,
                17371716718812495125,
                2287505336655193298,
                4396729111521177941,
                9109044948097956040,
                3549346631391027167,
                214618372
            ])
        );

        assert_eq!(
            b - a,
            fp_480::Fp480::new([
                9003500502228661940,
                7678524344415787604,
                13637545540568129179,
                10564581055598748841,
                13139757820151633100,
                8768206151349756512,
                5382499513376921553,
                3714284222
            ])
        );
        assert_eq!(
            a - b,
            fp_480::Fp480::new([
                17033269542773943779,
                11515467446333727102,
                11968236885165951752,
                9918736581825734195,
                8848378657505199061,
                10507666847254361432,
                14083357281300217175,
                580447141
            ])
        );
        assert_eq!(
            fp_480::Fp480::one() - a,
            fp_480::Fp480::new([
                13663488907129300739,
                15829417723857755103,
                7356286447732592834,
                13361909896326553579,
                4371514354315227579,
                18276324675335451852,
                916576440992947192,
                1749832925
            ])
        );
    }

    #[test]
    fn identity_single_case() {
        let b = fp_480::Fp480::new([
            3834872116353127868,
            4590656380457259402,
            9262714631161821576,
            12699347707666498727,
            8045166865395528593,
            11972162468224368488,
            7066104462708830816,
            3771584138,
        ]);
        let binverse = fp_480::Fp480::new([
            8359496029866312121,
            9855680935361061958,
            9721984913593910889,
            9699886586609381252,
            1495521600225809240,
            9137980985416018882,
            8475132370731740574,
            938930668,
        ]);
        assert_eq!(b.inv(), binverse);
        assert_eq!(b.mul(b.inv()), fp_480::Fp480::one());
        assert_eq!(b.div(b), fp_480::Fp480::one());

        let b = fp_480::Fp480::new([
            0xf94961046cc2343d,
            0x54e7fe7a0c15bf98,
            0x92c968b7a5dc19e6,
            0x134c24513c8e50bf,
            0xdd5d0558255e2bfe,
            0xe2e7b5df97c40138,
            0x784c26b25d56ca9b,
            0xec1d97d8,
        ]);
        let binverse = fp_480::Fp480::new([
            3964041005206816693,
            17748359409173047324,
            8574914009989265896,
            11108665920023441573,
            3240235572120937559,
            17453993893625142484,
            4629261177586478425,
            3466826655,
        ]);
        assert_eq!(b.inv(), binverse);
        assert_eq!(b.mul(b.inv()), fp_480::Fp480::one());
        assert_eq!(b.div(b), fp_480::Fp480::one());
    }

    #[test]
    fn div_single_case() {
        let a = fp_480::Fp480::new([
            2816712066946153747,
            5747042474327040776,
            15535406302961972262,
            16460818219144805138,
            6994911687676264632,
            12521802401546148605,
            9581905044881715501,
            3587491159,
        ]);
        let ainv = fp_480::Fp480::new([
            13654897055858194021,
            3339211318812279773,
            6858620690097655194,
            8897541996364047054,
            15724927113891581668,
            13694193647356111498,
            18416317810762613951,
            2200872997,
        ]);
        assert_eq!(a.inv(), ainv);

        let numerator = fp_480::Fp480::new([
            8348794426534327563,
            1879903001149673731,
            12837080870514459017,
            700911462630613964,
            9433880464073972719,
            1278767469094518492,
            18251606839666179829,
            2954588469,
        ]);
        let result = fp_480::Fp480::new([
            17013347194150995858,
            12466908776327473270,
            2481475576127098196,
            16975279220818532667,
            17740556660822707130,
            11848543690922999332,
            14202304424715009844,
            2162127586,
        ]);

        // numerator / a = result
        assert_eq!(numerator / a, result);

        let numerator = fp_480::Fp480::new([
            5173569617009464247,
            3223239650780825717,
            11226163026039631204,
            4560913629967781597,
            14321350974822713880,
            1294566703500450391,
            2384188416395534881,
            3289683177,
        ]);
        let denominator = fp_480::Fp480::new([
            3124894097069768307,
            14602323645599167979,
            13720940325689005600,
            11251012007713449105,
            3928127865078497824,
            2336440514434006119,
            8720211081311112793,
            2171615668,
        ]);
        let expected = fp_480::Fp480::new([
            145916248207258317,
            3571538994930808453,
            15848624085546498523,
            18371007075928284529,
            622524464600606627,
            14215098719906011444,
            4689483692729775607,
            3264262622,
        ]);

        let denominator_inverse = fp_480::Fp480::new([
            5194245210418602010,
            10809886684582169141,
            5666673393015102841,
            7341667117682842000,
            15709100179662702910,
            12435149750834608475,
            3861253510626063302,
            2159848540,
        ]);
        assert_eq!(denominator.inv(), denominator_inverse);
        assert_eq!(numerator / denominator, expected);

        // failure case from mul_equals_div
        let a = fp_480::Fp480::new([
            0x63b7da3a4e2d8e00,
            0x7e0d4680b329bb99,
            0x72d8a3f862f3fd88,
            0x53eda8ce171d68b3,
            0x7a2131869ec4ed19,
            0x59da4f56f5c945d2,
            0x85ae5a2efaf37c1d,
            0xbfc67cea,
        ]);
        let b = fp_480::Fp480::new([
            0x16ab759419cc4100,
            0x5014fe9dccb413a5,
            0x65493b2c5cabcdf7,
            0x154666ed0538f953,
            0x59cba9f0f5f2e84b,
            0x8d1a5d247b179226,
            0x731a78943d9db123,
            0xd02cf669,
        ]);
        let c = fp_480::Fp480::new([
            11693783694811866307,
            245594065984987110,
            322736986052945757,
            2942266530865397599,
            17072191670499601912,
            15567716623030039384,
            10534201835020433733,
            2741051863,
        ]);
        assert_eq!(a * b, c);
        assert_eq!(c / b, a);
        assert_eq!(c / a, b);
    }
    #[test]
    fn barrett_reduction() {
        // max
        let yuuuuge = [
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0x0000000000000000,
        ]; // full 480*2 bits of f's
        assert_eq!(
            fp_480::reduce_barrett(&yuuuuge),
            [
                12632987041438024573,
                4408161087846440449,
                911295513271048361,
                5586220749226910812,
                2691267771484745119,
                10927224045810247897,
                2993591145872866735,
                3014789011
            ]
        );

        let max = [
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFE,
        ]; // should not have all FF's as that is more than p^2, and also would require
           // a special case to handle.
        assert_eq!(
            fp_256::reduce_barrett(&max),
            [
                7259975683385058220,
                6648114513626429059,
                7748080958034249677,
                6672934772539367871
            ]
        );

        // 2*p = 0
        let twop = [
            0xd2aa60d07dd2c92e,
            0x14bd83addac6f7a6,
            0xc6b406e6e3a3d348,
            0x3886b9bb605c15ba,
            0x624b1b41af16b144,
            0x170350ba82af2f72,
            0x1c493b3d8eb5aa52,
            0x1fff8ccc8,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ]; // padded to be DOUBLENUMLIMBS long
        println!("{:?}", fp_480::reduce_barrett(&twop));
        assert!(fp_480::reduce_barrett(&twop).iter().all(|limb| *limb == 0));

        // p * p = 0
        // (p-5)*(p-4) = p^2 - 4p - 5p + 20 -- any multiple of p is zero, so = 20

        // Arbitrary big number x < p:
        // 1207811257040831621391376042066624303968527700717084310097981686745089556997378024160906781595716428319582984800146428900609196302855174292951579
        // x * x =
        // 1458808032634553832917799193961797434235419188246642318697058264038385049089265036191575901553853582615536794361502971074563184673635362176350783892004837696732862760694621978178376129161721367027911973240921051380566691956112820757574780900407857583461168170860656255046949060119638593241
        // x * x mod p =
        // 1809287164707112312378003899038518547656481275056607132587933524119638354881446495182091979711134781179141557515395208880017070421464511373264455
        let xsquared = [
            0xbea8647dcae44ed9,
            0xba4c1ef45853da22,
            0x72375b49df5d97fb,
            0x2c2e0cde58177c4c,
            0x20966564139cc478,
            0x984aeb6446ce6eb5,
            0xf9e1fdbcdf161900,
            0x383f508f2badf79a,
            0x15dad7130e760429,
            0x1b65bfc712f63ad2,
            0xd0c5f652604479d6,
            0x686175540020f57e,
            0xaa40c8a782f607ee,
            0xae42529ee1b07c42,
            0x26524c840aa5555e,
            0,
        ];
        let expected = [
            0xd3d1fd35b3ce4647,
            0x46ab040074bafa93,
            0x04d03e428eb12074,
            0xc256b919952dba44,
            0xc29d363dad222ddb,
            0xbc6ac9b19f947a5b,
            0xc6f716c70741a297,
            0x945f059c,
        ];
        assert_eq!(fp_480::reduce_barrett(&xsquared), expected);
        let x = [
            2677753075732170326,
            2697215944595530698,
            5179027707304142261,
            7384595457393154488,
            2325075600308826600,
            755475514309524552,
            5385728672330518784,
            481480908,
        ];
        let expected = [
            14071300896741445182,
            4673512632956559786,
            13968087039845073466,
            10355184993072047808,
        ];
        assert_eq!(fp_256::reduce_barrett(&x), expected);
    }

    #[test]
    fn debug_output_test256() {
        let other = fp_256::Fp256::new([0, 0, 0x00FFFFFFFFFFFFFFu64, 0]);
        let str = format!("debug: {:?}, decimal: {}, hex: {:x}", other, other, other);
        assert_eq!(&str.replace(" ", ""), "debug:Fp256(0x0,0x0,0xffffffffffffff,0x0),decimal:24519928653854221393451185513466483474525218523169423360,hex:0x000000000000000000ffffffffffffff00000000000000000000000000000000");
    }

    #[test]
    fn neg_test256() {
        use digits::util::*;
        let a = fp_256::Fp256::one();
        let b = fp_256::Fp256::new([
            6130910713169823785,
            12501959402729280499,
            16759952019381344743,
            857710904,
        ]);
        assert_eq!(a * b, b);
        println!("-a * b (limbs)={:?}", (*(-a)).mul_classic(&(*b)[..]));
        assert_eq!(-a * b, -b);
    }

    #[test]
    fn concrete_dist_assoc_norm() {
        let a = fp_256::Fp256::new([
            0xb12fb5043850e628,
            0xd7bab085515748e0,
            0x9b7355be1d204ae6,
            0x860ac8b5,
        ]);
        let b = fp_256::Fp256::new([
            0x2123ef649ef49cd4,
            0x8da4e712c8bdff77,
            0xb3799bf0e3bd329c,
            0x926e001c,
        ]);
        let c = fp_256::Fp256::new([
            10773964068678169461,
            7781056686061812801,
            7705577838994800718,
            91624696780573940,
        ]);
        assert_eq!(a * b, c);
        assert_eq!(c / a, b);
        assert_eq!(c * a.inv(), b);

        let a = fp_256::Fp256::new([
            0xb8b64011623c018a,
            0xddb38c295bc94eba,
            0x6cd16ea46cd6bd8f,
            0xd4d302ca,
        ]);
        let b = fp_256::Fp256::new([
            0x523d205e45ad60fc,
            0x21508a6d679848b0,
            0xf36e0f4429ce3982,
            0xf22e03e8,
        ]);
        let c = fp_256::Fp256::new([
            0x99343d8586c2db10,
            0x1dd26e85bda07d10,
            0xa59686b61a629965,
            0xd7cf30d3,
        ]);
        let expected = fp_256::Fp256::new([
            15795495403639475627,
            12400987731118697860,
            2558584020451078739,
            8223335412293341181,
        ]);
        let x = [
            17550962939841192978,
            11129715693094986530,
            14839879005967945333,
            131776332513548278,
        ]; // this has a most sig of 1 not shown; mod of this should be 'expected'

        let fpx = fp_256::Fp256::new(x);
        println!("\n\n\n\n***** about to call normalize\n");
        assert_eq!(fpx.normalize(1), expected);

        assert_eq!(
            fp_256::Fp256::new([
                1658067930733296012,
                13547302404868533267,
                9458636050195978030,
                9117568210736327771
            ]) + fp_256::Fp256::new([
                15892895009107896966,
                16029157361936004879,
                5381242955771967302,
                9460952195486772123
            ]),
            expected
        );
        assert_eq!(a * (b + c), a * b + a * c);
        assert_eq!(a * (b + c), expected);

        let max = fp_256::Fp256::new([
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
            0xFFFFFFFFFFFFFFFF,
        ]);
        assert_eq!(
            max.normalize(0),
            fp_256::Fp256::new([
                16691276537507834264,
                1271272038023711329,
                6165449088192685022,
                8091559079779792902
            ])
        );
        assert_eq!(
            max.normalize(1),
            fp_256::Fp256::new([
                13180341465104399562,
                3813816114071133989,
                49603190868503450,
                5827933165629827091
            ])
        );
    }

    #[test]
    fn test_from_sha_static() {
        let x = [1u8; 64];
        let expected = fp_256::Fp256::new([
            9860606201530643810,
            16492608404628923632,
            13807063077430300195,
            8441074048361149479,
        ]);
        assert_eq!(fp_256::Fp256::from(x), expected);

        let mut x = [0u8; 64];
        x[16..32].iter_mut().for_each(|i| *i = 1);
        x[48..64].iter_mut().for_each(|i| *i = 1);
        let expected = fp_256::Fp256::new([
            7418543689605779358,
            14139341254799044026,
            1629308205113762586,
            8686893989492885021,
        ]);
        assert_eq!(fp_256::Fp256::from(x), expected);
    }

    #[test]
    fn fp256_to_bytes_known_good_value() {
        use fp_256::Fp256;
        let fp = Fp256::from(255);
        let bytes = fp.to_bytes_array();
        let expected_result = {
            let mut array = [0u8; 32];
            array[31] = 255;
            array
        };
        assert_eq!(bytes, expected_result);
    }

    #[test]
    fn fp256_from_bytes_should_mod() {
        use fp_256::Fp256;
        let max_bytes = Fp256::from([255u8; 32]);
        let result = max_bytes.to_bytes_array();
        assert_eq!(
            result,
            [
                112, 74, 254, 28, 181, 92, 120, 6, 85, 144, 19, 71, 158, 123, 35, 222, 17, 164,
                119, 46, 223, 74, 74, 97, 231, 163, 83, 147, 161, 247, 105, 152
            ]
        );
    }
}
