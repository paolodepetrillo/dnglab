use byteorder::{BigEndian, ReadBytesExt};
use num_enum::TryFromPrimitive;
use std::io::{Cursor, Error, ErrorKind, Result, Seek, SeekFrom};

#[derive(Copy, Clone, TryFromPrimitive)]
#[repr(u32)]
pub enum DngOpcodeId {
  WarpRectilinear = 1,
  WarpFisheye = 2,
  FixVignetteRadial = 3,
  FixBadPixelsConstant = 4,
  FixBadPixelsList = 5,
  TrimBounds = 6,
  MapTable = 7,
  MapPolynomial = 8,
  GainMap = 9,
  DeltaPerRow = 10,
  DeltaPerColumn = 11,
  ScalePerRow = 12,
  ScalePerColumn = 13,
  WarpRectilinear2 = 14,
}

#[derive(Debug)]
pub struct DngOpcodeFlags {
  pub optional: bool,
  pub preview_skip: bool,
}

impl DngOpcodeFlags {
  fn decode(v: u32) -> Self {
    DngOpcodeFlags {
      optional: v & 1 > 0,
      preview_skip: v & 2 > 0,
    }
  }
}

#[derive(Debug)]
pub struct DngOpcodeRegion {
  pub top: u32,
  pub left: u32,
  pub bottom: u32,
  pub right: u32,
  pub plane: u32,
  pub planes: u32,
  pub row_pitch: u32,
  pub col_pitch: u32,
}

impl DngOpcodeRegion {
  fn decode(cur: &mut Cursor<&[u8]>) -> Result<DngOpcodeRegion> {
    Ok(DngOpcodeRegion {
      top: cur.read_u32::<BigEndian>()?,
      left: cur.read_u32::<BigEndian>()?,
      bottom: cur.read_u32::<BigEndian>()?,
      right: cur.read_u32::<BigEndian>()?,
      plane: cur.read_u32::<BigEndian>()?,
      planes: cur.read_u32::<BigEndian>()?,
      row_pitch: cur.read_u32::<BigEndian>()?,
      col_pitch: cur.read_u32::<BigEndian>()?,
    })
  }
}

#[derive(Debug)]
pub struct WarpRectilinearCoef {
  pub kr: [f64; 4],
  pub kt: [f64; 2],
}

impl WarpRectilinearCoef {
  fn decode(cur: &mut Cursor<&[u8]>) -> Result<WarpRectilinearCoef> {
    let mut kr = vec![0.0; 4];
    cur.read_f64_into::<BigEndian>(&mut kr)?;
    let mut kt = vec![0.0; 2];
    cur.read_f64_into::<BigEndian>(&mut kt)?;
    Ok(WarpRectilinearCoef {
      kr: kr.try_into().unwrap(),
      kt: kt.try_into().unwrap(),
    })
  }
}

#[derive(Debug)]
pub struct WarpRectilinear {
  pub flags: DngOpcodeFlags,
  pub center_x: f64,
  pub center_y: f64,
  pub coefs: Vec<WarpRectilinearCoef>,
}

impl WarpRectilinear {
  fn decode(flags: DngOpcodeFlags, cur: &mut Cursor<&[u8]>) -> Result<WarpRectilinear> {
    let n = cur.read_u32::<BigEndian>()? as usize;
    let coefs = (0..n).map(|_| WarpRectilinearCoef::decode(cur)).collect::<Result<Vec<WarpRectilinearCoef>>>()?;
    let center_x = cur.read_f64::<BigEndian>()?;
    let center_y = cur.read_f64::<BigEndian>()?;
    Ok(WarpRectilinear {
      flags,
      coefs,
      center_x,
      center_y,
    })
  }
}

#[derive(Debug)]
pub struct WarpFisheyeCoef {
  pub kr: [f64; 4],
}

impl WarpFisheyeCoef {
  fn decode(cur: &mut Cursor<&[u8]>) -> Result<WarpFisheyeCoef> {
    let mut kr = vec![0.0; 4];
    cur.read_f64_into::<BigEndian>(&mut kr)?;
    Ok(WarpFisheyeCoef { kr: kr.try_into().unwrap() })
  }
}

#[derive(Debug)]
pub struct WarpFisheye {
  pub flags: DngOpcodeFlags,
  pub center_x: f64,
  pub center_y: f64,
  pub coefs: Vec<WarpFisheyeCoef>,
}

impl WarpFisheye {
  fn decode(flags: DngOpcodeFlags, cur: &mut Cursor<&[u8]>) -> Result<WarpFisheye> {
    let n = cur.read_u32::<BigEndian>()? as usize;
    let coefs = (0..n).map(|_| WarpFisheyeCoef::decode(cur)).collect::<Result<Vec<WarpFisheyeCoef>>>()?;
    let center_x = cur.read_f64::<BigEndian>()?;
    let center_y = cur.read_f64::<BigEndian>()?;
    Ok(WarpFisheye {
      flags,
      coefs,
      center_x,
      center_y,
    })
  }
}

#[derive(Debug)]
pub struct FixVignetteRadial {
  pub flags: DngOpcodeFlags,
  pub k: [f64; 5],
  pub center_x: f64,
  pub center_y: f64,
}

impl FixVignetteRadial {
  fn decode(flags: DngOpcodeFlags, cur: &mut Cursor<&[u8]>) -> Result<FixVignetteRadial> {
    let mut k = vec![0.0; 5];
    cur.read_f64_into::<BigEndian>(&mut k)?;
    let center_x = cur.read_f64::<BigEndian>()?;
    let center_y = cur.read_f64::<BigEndian>()?;
    Ok(FixVignetteRadial {
      flags,
      k: k.try_into().unwrap(),
      center_x,
      center_y,
    })
  }
}

#[derive(Debug)]
pub struct FixBadPixelsConstant {
  pub flags: DngOpcodeFlags,
  pub constant: u32,
  pub bayer_phase: u32,
}

impl FixBadPixelsConstant {
  fn decode(flags: DngOpcodeFlags, cur: &mut Cursor<&[u8]>) -> Result<FixBadPixelsConstant> {
    Ok(FixBadPixelsConstant {
      flags,
      constant: cur.read_u32::<BigEndian>()?,
      bayer_phase: cur.read_u32::<BigEndian>()?,
    })
  }
}

#[derive(Debug)]
pub struct BadPoint {
  pub row: u32,
  pub column: u32,
}

impl BadPoint {
  fn decode(cur: &mut Cursor<&[u8]>) -> Result<BadPoint> {
    Ok(BadPoint {
      row: cur.read_u32::<BigEndian>()?,
      column: cur.read_u32::<BigEndian>()?,
    })
  }
}

#[derive(Debug)]
pub struct BadRect {
  pub top: u32,
  pub left: u32,
  pub bottom: u32,
  pub right: u32,
}

impl BadRect {
  fn decode(cur: &mut Cursor<&[u8]>) -> Result<BadRect> {
    Ok(BadRect {
      top: cur.read_u32::<BigEndian>()?,
      left: cur.read_u32::<BigEndian>()?,
      bottom: cur.read_u32::<BigEndian>()?,
      right: cur.read_u32::<BigEndian>()?,
    })
  }
}

#[derive(Debug)]
pub struct FixBadPixelsList {
  pub flags: DngOpcodeFlags,
  pub bayer_phase: u32,
  pub bad_points: Vec<BadPoint>,
  pub bad_rects: Vec<BadRect>,
}

impl FixBadPixelsList {
  fn decode(flags: DngOpcodeFlags, cur: &mut Cursor<&[u8]>) -> Result<FixBadPixelsList> {
    let bayer_phase = cur.read_u32::<BigEndian>()?;
    let num_points = cur.read_u32::<BigEndian>()?;
    let num_rects = cur.read_u32::<BigEndian>()?;
    let bad_points = (0..num_points).map(|_| BadPoint::decode(cur)).collect::<Result<Vec<BadPoint>>>()?;
    let bad_rects = (0..num_rects).map(|_| BadRect::decode(cur)).collect::<Result<Vec<BadRect>>>()?;
    Ok(FixBadPixelsList {
      flags,
      bayer_phase,
      bad_points,
      bad_rects,
    })
  }
}

#[derive(Debug)]
pub struct TrimBounds {
  pub flags: DngOpcodeFlags,
  pub top: u32,
  pub left: u32,
  pub bottom: u32,
  pub right: u32,
}

impl TrimBounds {
  fn decode(flags: DngOpcodeFlags, cur: &mut Cursor<&[u8]>) -> Result<TrimBounds> {
    Ok(TrimBounds {
      flags,
      top: cur.read_u32::<BigEndian>()?,
      left: cur.read_u32::<BigEndian>()?,
      bottom: cur.read_u32::<BigEndian>()?,
      right: cur.read_u32::<BigEndian>()?,
    })
  }
}

#[derive(Debug)]
pub struct MapTable {
  pub flags: DngOpcodeFlags,
  pub region: DngOpcodeRegion,
  pub table: Vec<u16>,
}

impl MapTable {
  fn decode(flags: DngOpcodeFlags, cur: &mut Cursor<&[u8]>) -> Result<MapTable> {
    let region = DngOpcodeRegion::decode(cur)?;
    let len: usize = cur.read_u32::<BigEndian>()? as usize;
    let mut table = vec![0; len];
    cur.read_u16_into::<BigEndian>(&mut table)?;
    Ok(MapTable { flags, region, table })
  }
}

#[derive(Debug)]
pub struct MapPolynomial {
  pub flags: DngOpcodeFlags,
  pub region: DngOpcodeRegion,
  pub coefs: Vec<f64>,
}

impl MapPolynomial {
  fn decode(flags: DngOpcodeFlags, cur: &mut Cursor<&[u8]>) -> Result<MapPolynomial> {
    let region = DngOpcodeRegion::decode(cur)?;
    let degree: usize = cur.read_u32::<BigEndian>()? as usize;
    let mut coefs = vec![0.0; degree + 1];
    cur.read_f64_into::<BigEndian>(&mut coefs)?;
    Ok(MapPolynomial { flags, region, coefs })
  }
}

#[derive(Debug)]
pub struct GainMap {
  pub flags: DngOpcodeFlags,
  pub region: DngOpcodeRegion,
  pub map_points_v: u32,
  pub map_points_h: u32,
  pub map_spacing_v: f64,
  pub map_spacing_h: f64,
  pub map_origin_v: f64,
  pub map_origin_h: f64,
  pub map_planes: u32,
  pub map_gain: Vec<f32>,
}

impl GainMap {
  fn decode(flags: DngOpcodeFlags, cur: &mut Cursor<&[u8]>) -> Result<GainMap> {
    let region = DngOpcodeRegion::decode(cur)?;
    let map_points_v = cur.read_u32::<BigEndian>()?;
    let map_points_h = cur.read_u32::<BigEndian>()?;
    let map_spacing_v = cur.read_f64::<BigEndian>()?;
    let map_spacing_h = cur.read_f64::<BigEndian>()?;
    let map_origin_v = cur.read_f64::<BigEndian>()?;
    let map_origin_h = cur.read_f64::<BigEndian>()?;
    let map_planes = cur.read_u32::<BigEndian>()?;
    let len = (map_points_h * map_points_v * map_planes) as usize;
    let mut map_gain = vec![0.0; len];
    cur.read_f32_into::<BigEndian>(&mut map_gain)?;
    Ok(GainMap {
      flags,
      region,
      map_points_v,
      map_points_h,
      map_spacing_v,
      map_spacing_h,
      map_origin_v,
      map_origin_h,
      map_planes,
      map_gain,
    })
  }
}

#[derive(Debug)]
pub struct WarpRectilinear2Coef {
  pub kr: [f64; 15],
  pub kt: [f64; 2],
}

impl WarpRectilinear2Coef {
  fn decode(cur: &mut Cursor<&[u8]>) -> Result<WarpRectilinear2Coef> {
    let mut kr = vec![0.0; 15];
    cur.read_f64_into::<BigEndian>(&mut kr)?;
    let mut kt = vec![0.0; 2];
    cur.read_f64_into::<BigEndian>(&mut kt)?;
    Ok(WarpRectilinear2Coef {
      kr: kr.try_into().unwrap(),
      kt: kt.try_into().unwrap(),
    })
  }
}

#[derive(Debug)]
pub struct WarpRectilinear2 {
  pub flags: DngOpcodeFlags,
  pub center_x: f64,
  pub center_y: f64,
  pub reciprocal_radial: u32,
  pub coefs: Vec<WarpRectilinear2Coef>,
}

impl WarpRectilinear2 {
  fn decode(flags: DngOpcodeFlags, cur: &mut Cursor<&[u8]>) -> Result<WarpRectilinear2> {
    let n = cur.read_u32::<BigEndian>()? as usize;
    let coefs = (0..n)
      .map(|_| WarpRectilinear2Coef::decode(cur))
      .collect::<Result<Vec<WarpRectilinear2Coef>>>()?;
    let center_x = cur.read_f64::<BigEndian>()?;
    let center_y = cur.read_f64::<BigEndian>()?;
    let reciprocal_radial = cur.read_u32::<BigEndian>()?;
    Ok(WarpRectilinear2 {
      flags,
      coefs,
      center_x,
      center_y,
      reciprocal_radial,
    })
  }
}

#[derive(Debug)]
pub struct ValuesPerRowOrCol {
  pub flags: DngOpcodeFlags,
  pub region: DngOpcodeRegion,
  pub values: Vec<f32>,
}

impl ValuesPerRowOrCol {
  fn decode(flags: DngOpcodeFlags, cur: &mut Cursor<&[u8]>) -> Result<ValuesPerRowOrCol> {
    let region = DngOpcodeRegion::decode(cur)?;
    let len: usize = cur.read_u32::<BigEndian>()? as usize;
    let mut values = vec![0.0; len];
    cur.read_f32_into::<BigEndian>(&mut values)?;
    Ok(ValuesPerRowOrCol { flags, region, values })
  }
}

#[derive(Debug)]
pub enum DngOpcode {
  WarpRectilinear(WarpRectilinear),
  WarpFisheye(WarpFisheye),
  FixVignetteRadial(FixVignetteRadial),
  FixBadPixelsConstant(FixBadPixelsConstant),
  FixBadPixelsList(FixBadPixelsList),
  TrimBounds(TrimBounds),
  MapTable(MapTable),
  MapPolynomial(MapPolynomial),
  GainMap(GainMap),
  DeltaPerRow(ValuesPerRowOrCol),
  DeltaPerColumn(ValuesPerRowOrCol),
  ScalePerRow(ValuesPerRowOrCol),
  ScalePerColumn(ValuesPerRowOrCol),
  WarpRectilinear2(WarpRectilinear2),
}

pub fn decode_opcode_list(opcode_list: &[u8]) -> Result<Vec<DngOpcode>> {
  let mut cur = Cursor::new(opcode_list);
  let mut ops = Vec::new();

  let count = cur.read_u32::<BigEndian>()?;
  for _ in 0..count {
    let op_id_code = cur.read_u32::<BigEndian>()?;
    let _op_spec_ver = cur.read_u32::<BigEndian>()?;
    let op_flags = cur.read_u32::<BigEndian>()?;
    let op_len = cur.read_u32::<BigEndian>()?;
    let pos_start = cur.position() as u32;

    match DngOpcodeId::try_from(op_id_code) {
      Ok(op_id) => {
        let flags = DngOpcodeFlags::decode(op_flags);
        let op = match op_id {
          DngOpcodeId::WarpRectilinear => DngOpcode::WarpRectilinear(WarpRectilinear::decode(flags, &mut cur)?),
          DngOpcodeId::WarpFisheye => DngOpcode::WarpFisheye(WarpFisheye::decode(flags, &mut cur)?),
          DngOpcodeId::FixVignetteRadial => DngOpcode::FixVignetteRadial(FixVignetteRadial::decode(flags, &mut cur)?),
          DngOpcodeId::FixBadPixelsConstant => DngOpcode::FixBadPixelsConstant(FixBadPixelsConstant::decode(flags, &mut cur)?),
          DngOpcodeId::FixBadPixelsList => DngOpcode::FixBadPixelsList(FixBadPixelsList::decode(flags, &mut cur)?),
          DngOpcodeId::TrimBounds => DngOpcode::TrimBounds(TrimBounds::decode(flags, &mut cur)?),
          DngOpcodeId::MapTable => DngOpcode::MapTable(MapTable::decode(flags, &mut cur)?),
          DngOpcodeId::MapPolynomial => DngOpcode::MapPolynomial(MapPolynomial::decode(flags, &mut cur)?),
          DngOpcodeId::GainMap => DngOpcode::GainMap(GainMap::decode(flags, &mut cur)?),
          DngOpcodeId::DeltaPerRow => DngOpcode::DeltaPerRow(ValuesPerRowOrCol::decode(flags, &mut cur)?),
          DngOpcodeId::DeltaPerColumn => DngOpcode::DeltaPerColumn(ValuesPerRowOrCol::decode(flags, &mut cur)?),
          DngOpcodeId::ScalePerRow => DngOpcode::ScalePerRow(ValuesPerRowOrCol::decode(flags, &mut cur)?),
          DngOpcodeId::ScalePerColumn => DngOpcode::ScalePerColumn(ValuesPerRowOrCol::decode(flags, &mut cur)?),
          DngOpcodeId::WarpRectilinear2 => DngOpcode::WarpRectilinear2(WarpRectilinear2::decode(flags, &mut cur)?),
        };
        if pos_start + op_len != cur.position() as u32 {
          return Err(Error::new(ErrorKind::Other, "Invalid opcode size"));
        }
        ops.push(op);
      }
      Err(_e) => {
        // Unsupported opcode, skip it
        _ = cur.seek(SeekFrom::Current(op_len.into()))
      }
    }
  }

  Ok(ops)
}
