use ark_ec::{pairing::{self, Pairing, PairingOutput}, Group, mnt6::G1Projective, CurveGroup, CurveConfig};
use ark_ec::short_weierstrass::{SWCurveConfig, Projective};
use ark_ff::{MontFp, Zero, BigInt, CubicExtField, QuadExtField, BigInteger320, utils, Field, QuadExtConfig};

use serde::{Deserialize, ser::StdError};
use std::{fs, env};

use core::str::FromStr;


use anyhow::{anyhow, Result};

/*
use ark_mnt6_298::{
    fq::Fq, fq3::Fq3, fr::Fr, Fq6, G1Projective as G1, G2Projective as G2, MNT6_298,
    G1Affine as G1a,
    G2Affine as G2a,
};
*/

use ark_bn254::{
    fq::Fq, fq2::Fq2, fr::Fr, Fq6, Fq12,
    G1Projective as G1, G2Projective as G2, Bn254,
    G1Affine as G1a,
    G2Affine as G2a,
};

/*
enum curve_operation_test_points : std::size_t {
    p1,
    p2,
    p1_plus_p2,
    p1_minus_p2,
    p1_mul_C1,
    p2_mul_C1_plus_p2_mul_C2,
    p1_dbl,
    p1_mixed_add_p2,
    p1_to_affine,
    p2_to_special
};
*/

#[derive(Deserialize, Debug)]
struct ProjectivePointG1(Vec<String>);

#[derive(Deserialize, Debug)]
struct ProjectivePointG2(Vec<(String,String)>);

#[derive(Deserialize, Debug)]
struct TestSample<PP> {
    pub constants: Vec<u64>,
    pub point_coordinates: Vec<PP>,
}

fn g1_from_sample(x: &ProjectivePointG1) -> Result<Projective<ark_bn254::g1::Config>>
{
    Ok( Projective {
        x: x.0[0].parse().map_err(|_| anyhow!("failed to parse x"))?,
        y: x.0[1].parse().map_err(|_| anyhow!("failed to parse y"))?,
        z: x.0[2].parse().map_err(|_| anyhow!("failed to parse z"))?,
    })
}

fn g2_from_sample(x: &ProjectivePointG2) -> Result<Projective<ark_bn254::g2::Config>>
{
    Ok( Projective {
        x: QuadExtField {
            c0: x.0[0].0.parse().map_err(|_| anyhow!("failed to parse x.c0"))?,
            c1: x.0[0].1.parse().map_err(|_| anyhow!("failed to parse x.c1"))?,
        },
        y: QuadExtField {
            c0: x.0[1].0.parse().map_err(|_| anyhow!("failed to parse y.c0"))?,
            c1: x.0[1].1.parse().map_err(|_| anyhow!("failed to parse y.c1"))?,
        },
        z: QuadExtField {
            c0: x.0[2].0.parse().map_err(|_| anyhow!("failed to parse z.c0"))?,
            c1: x.0[2].1.parse().map_err(|_| anyhow!("failed to parse z.c1"))?,
        },
    })
}


#[derive(Debug)]
struct TestData<P:SWCurveConfig> {
    pub c1: P::ScalarField,
    pub c2: P::ScalarField,
    pub p1: Projective<P>,
    pub p2: Projective<P>,
    pub p1_plus_p2: Projective<P>,
    pub p1_minus_p2: Projective<P>,
    pub p1_mul_c1: Projective<P>,
    pub p2_mul_c1_plus_p2_mul_c2: Projective<P>,
    pub p1_dbl: Projective<P>,
}

impl TestData<ark_bn254::g1::Config> {
    fn from(sample: &TestSample<ProjectivePointG1>) -> Result<Self> {
        Ok(TestData {
            c1: self::Fr::from(sample.constants[0]),
            c2: self::Fr::from(sample.constants[1]),
            p1 : g1_from_sample(&sample.point_coordinates[0])?,
            p2 : g1_from_sample(&sample.point_coordinates[1])?,
            p1_plus_p2        : g1_from_sample(&sample.point_coordinates[2])?,
            p1_minus_p2       : g1_from_sample(&sample.point_coordinates[3])?,
            p1_mul_c1                : g1_from_sample(&sample.point_coordinates[4])?,
            p2_mul_c1_plus_p2_mul_c2 : g1_from_sample(&sample.point_coordinates[5])?,
            p1_dbl                   : g1_from_sample(&sample.point_coordinates[6])?,
        })
    }
}

impl TestData<ark_bn254::g2::Config> {
    fn from(sample: &TestSample<ProjectivePointG2>) -> Result<Self> {
        Ok(TestData {
            c1: self::Fr::from(sample.constants[0]),
            c2: self::Fr::from(sample.constants[1]),
            p1 : g2_from_sample(&sample.point_coordinates[0])?,
            p2 : g2_from_sample(&sample.point_coordinates[1])?,
            p1_plus_p2        : g2_from_sample(&sample.point_coordinates[2])?,
            p1_minus_p2       : g2_from_sample(&sample.point_coordinates[3])?,
            p1_mul_c1                : g2_from_sample(&sample.point_coordinates[4])?,
            p2_mul_c1_plus_p2_mul_c2 : g2_from_sample(&sample.point_coordinates[5])?,
            p1_dbl                   : g2_from_sample(&sample.point_coordinates[6])?,
        })
    }
}


fn print_projective<P: SWCurveConfig>(label: &str, p: &Projective<P>) {
    println!("{}: inf: {}", label, p.is_zero());
    println!("X: {}", p.x);
    println!("Y: {}", p.y);
    println!("Z: {}", p.z);
}

fn run_test_case<P: SWCurveConfig>(data: &TestData<P>) -> Result<()> {

    assert!( data.p1+data.p2         == data.p1_plus_p2 );
    assert!( data.p1-data.p2         == data.p1_minus_p2 );
    assert!( data.p1*data.c1         == data.p1_mul_c1 );
    assert!( data.p2*data.c1 + data.p2*data.c2 == data.p2_mul_c1_plus_p2_mul_c2 );
    assert!( data.p1 + data.p1 == data.p1_dbl );
 
    Ok(())
}

fn main() -> Result<()> {
    let sample_g1_str = fs::read_to_string("bn128_g1.json")?;
    let sample_g2_str = fs::read_to_string("bn128_g2.json")?;

    let sample_g1 : TestSample::<ProjectivePointG1> = serde_json::from_str(&sample_g1_str)?;
    let sample_g2 : TestSample::<ProjectivePointG2> = serde_json::from_str(&sample_g2_str)?;

    let sg1 = TestData::<ark_bn254::g1::Config>::from(&sample_g1)?;
    let sg2 = TestData::<ark_bn254::g2::Config>::from(&sample_g2)?;

    println!("Running test case g1");
    run_test_case(&sg1)?;
    println!("Running test case g2");
    run_test_case(&sg2)?;

//    println!("{sg1:?}");
//    println!("{sg2:?}");

//    run_test_case_g1::<ark_bn254::g1::Config>(&sample_g1)?;
//    run_test_case_g2::<ark_bn254::g2::Config>(&sample_g2)?;

   /* 
    let sample : TestSample = serde_json::from_str(&sample_str)?;

    //println!("{sample:?}");

    let c1 = Fr::from(sample.constants[0]);
    let c2 = Fr::from(sample.constants[1]);

    let p1 = from_sample::<ark_bn254::g1::Config>(&sample.point_coordinates[0])?;
    let p2 = from_sample::<ark_bn254::g1::Config>(&sample.point_coordinates[1])?;
    let p1_plus_p2        = from_sample::<ark_bn254::g1::Config>(&sample.point_coordinates[2])?;
    let p1_minus_p2       = from_sample::<ark_bn254::g1::Config>(&sample.point_coordinates[3])?;
    let p1_mul_c1                = from_sample::<ark_bn254::g1::Config>(&sample.point_coordinates[4])?;
    let p2_mul_c1_plus_p2_mul_c2 = from_sample::<ark_bn254::g1::Config>(&sample.point_coordinates[5])?;
    let p1_dbl                   = from_sample::<ark_bn254::g1::Config>(&sample.point_coordinates[6])?;
   */
    /*
    let p1_mixed_add_p2   = from_sample(&sample.point_coordinates[7])?;
    let p1_to_affine      = from_sample(&sample.point_coordinates[8])?;
    let p2_to_specialmul_c2 = from_sample(&sample.point_coordinates[9])?;
    */
    /*
    print_projective("p1", &p1);
    print_projective("p2", &p2);

    println!("p1: {}", p1);
    println!("p2: {}", p2);

    println!("p1+p2: {}", p1_plus_p2);
    println!("p1-p2: {}", p1_minus_p2);


    assert!( p1+p2 == p1_plus_p2 );
    assert!( p1-p2 == p1_minus_p2 );
    assert!( p1*c1 == p1_mul_c1 );
    assert!( p2*c1 + p2*c2 == p2_mul_c1_plus_p2_mul_c2 );
    assert!( p1 + p1 == p1_dbl );
*/
 //   println!("a: {}", ap);
//    println!("b: {}", bp);


//    assert!( ap.into_affine() == a);
//    assert!( bp.into_affine() == b);



    /*

    let a = G1 {
        x: BigInt!("12269247201566734235290303960228304064272181820309912170654669102673702909924").into(),
        y: BigInt!("7534834744745566755779444323521770348604421192024675222326947617690858336843").into(),
        z: BigInt!("19659871856814291472586476920199797648479256735971090116560725328148931063076").into()
    };

    let b = G2 {
        x:  QuadExtField {
            c0: BigInt!("11522376354975166148131015333084519901972635889215132901625401307693742623612").into(),
            c1: BigInt!("19903118143127418197539921417218276500897631110464855011475958221668726352039").into()
        },
        y:  QuadExtField {
            c0: BigInt!("1344770348121910498932244667029986982789419063071610001587057752895730428925").into(),
            c1: BigInt!("2314690981825373512767849123313524417497870835736896866105938175119798276527").into()
        },
        z:  QuadExtField {
            c0: BigInt!("10903476179225791757910967430945084851993828984507567937937709713308956700413").into(),
            c1: BigInt!("4179030783092486396357085872012954999896912745373259683835662381653191084621").into()
        },
    };

    let c = G1 {
        x: BigInt!("17576187365457018637596854432164910741165446525364309881073995899323148890141").into(),
        y: BigInt!("15090934536779403392931323270381069965749897181056441674897976344629896928911").into(),
        z: BigInt!("11663274186732908459461053646736848618582236074377530157535156640397346916339").into(),
    };

    let vx = G1 {
        x: BigInt!("15137360228785888752929793295448335857645774743287765685182308256419951288058").into(),
        y: BigInt!("4916410736810946016577716260240046274867438480891067145233532059901769897865" ).into(),
        z: BigInt!("9498345608859154581964927860081729703664442556517890209350970418641244327291" ).into(),
    };
    let vy = G2 {
        x:  QuadExtField {
            c0: BigInt!("15742260173423840105876886327401473032241228078704241621299769298320239800751").into(),
            c1: BigInt!("9582760905507060200882049891612421435631925510727656593451666636456126510355" ).into()
        },
        y:  QuadExtField {
            c0: BigInt!("9216174655404894317474317502933311689833645708857363668766344536024814952416"). into(),
            c1: BigInt!("10301485467008915864545055661437507218004976213645779411016207515801156008661").into()
        },
        z:  QuadExtField {
            c0: BigInt!("21447687030484319319518448670456682409470849129912075242507876767652214839671").into(),
            c1: BigInt!("3583661946652261831756244346594638554252626017257641348927732483218611748085" ).into()
        },
    };

    let vz = G2 {
        x:  QuadExtField {
            c0: BigInt!("17028215694222747292376119808327826289678197373733289196686774972583389607267").into(),
            c1: BigInt!("18493282377719522345108356331705132997595597636236067387172563832259617184711").into()
        },
        y:  QuadExtField {
            c0: BigInt!("13529368367489722920611670686017761768640691196302375033894563592723315889319").into(),
            c1: BigInt!("10350786872763021215323950711261922508863396412884205000480105139693387580824").into()
        },
        z:  QuadExtField {
            c0: BigInt!("3846410746634373569159036485532245471676442303074351352583340272222115800935" ).into(),
            c1: BigInt!("8865582281903261939043162077749128945072482313552083132944865016128300307260" ).into()
        },
    };

    println!("A: {}", a);
    println!("B: {}", b);
    println!("C: {}", c);
    println!("vx: {}", vx);
    println!("vy: {}", vy);
    println!("vz: {}", vz);


    let ab = Bn254::pairing(a,b);
    let vv = Bn254::pairing(vx, vy);
    let cvz = Bn254::pairing(c, vz);

    let c_pairing = vv.0 * cvz.0;

    println!("ab: {}", ab);
    println!("vv: {}", vv);
    println!("cvz: {}", cvz);

    println!("c_pair: {}", c_pairing);

    println!("eq? {}", c_pairing == ab.0);

*/

    /*
    let g1a = G1a {
        x: BigInt!("384935640847912880393693533927607256225730398503834687089416162798619607759820524148671426").into(),
        y: BigInt!("314826256151918806891785023374026398102979423892073924067918611516917769322592664151809000").into(),
        infinity: true,
    };
    println!("g1ax: {}",g1a.x);
    println!("g1ay: {}",g1a.y);

    let g1ap = G1::from(g1a);

    let g2a = G2a {
        x: CubicExtField {
            c0: BigInt!("135414625673161647029542576796922445059173272027080877396520188364080489284363147418210349").into(),
            c1: BigInt!("292190435166420619744646442286808914331265238618078034880990899539567428142921431464978527").into(),
            c2: BigInt!("333085046202345096675841530310317719512359007484861515824043352535823071322502405004777").into()
        },
        y: CubicExtField {
            c0: BigInt!("16696375870924371998567822517128660031147400541258053061136695137492555882101439300584620").into(),
            c1: BigInt!("113024553072648467530424519430317907129041318971506060043094261140144556557168031437237450").into(),
            c2: BigInt!("297423169851021933417779789597770515397573494080582558555137441218775478797127567283064551").into(),
        },
        infinity: false,
    };

    let g2ap = G2::from(g2a);

    let gt = MNT6_298::pairing(g1ap, g2ap);
    println!("gt: {gt}");
    */

    Ok( () )
}
