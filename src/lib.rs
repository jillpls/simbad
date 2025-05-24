use std::fmt::{Debug, Display, Formatter};
use std::path::Path;
use csv::ReaderBuilder;
use glam::{DVec2, DVec3};
use serde::{Deserialize, Serialize};

#[derive(Default, Clone, Debug)]
pub struct Star {
    pub id: usize,
    pub pos: DVec3,
    pub name: String,
    pub class: String,
    pub constellation: String
}

#[derive(Default, Copy, Clone, Debug)]
pub struct StellarPosition {
    pub distance: f64,
    pub coord: EquatorialCoordinate
}

impl StellarPosition {
    pub fn new(distance: f64, right_ascension: f64, declination: f64) -> Self {
        Self {
            distance,
            coord : EquatorialCoordinate::new(right_ascension, declination)
        }
    }
}

impl Display for StellarPosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "dist: {:.2}, ra: {:.2}°, dec: {}{:.2}°", self.distance, self.coord.right_ascension.to_degrees(), if self.coord.declination >= 0. { "+"} else { ""},self.coord.declination.to_degrees())
    }
}

impl Into<DVec3> for StellarPosition {
    fn into(self) -> DVec3 {
        let adj = self.distance*self.coord.declination.cos();
        let opp = self.distance*self.coord.declination.sin();
        let plane_vec = DVec2::new(adj * self.coord.right_ascension.cos(), adj * self.coord.right_ascension.sin());
        DVec3::new(plane_vec.x, plane_vec.y, opp)
    }
}

impl From<DVec3> for StellarPosition {
    fn from(value: DVec3) -> Self {
        let hyp= value.length();
        let plane_vec = DVec2::new(value.x, value.y);
        let adj = plane_vec.length();
        let dec = if hyp != 0. {(adj/hyp).acos() * value.z.signum()} else { 0. };
        let ra = (plane_vec.y.abs()).atan2(plane_vec.x.abs());
        let x = plane_vec.x;
        let y = plane_vec.y;
        let ra = if x >= 0. && y >= 0. { ra } else if x >= 0. && y <= 0. { 180f64.to_radians() -ra } else if x <= 0. && y <= 0. { 180f64.to_radians() + ra} else { 360f64.to_radians() - ra };
        Self {
            distance: hyp,
            coord: EquatorialCoordinate {
                right_ascension: ra,
                declination: dec,
            },
        }
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct EquatorialCoordinate {
    pub right_ascension: f64,
    pub declination: f64
}


impl EquatorialCoordinate {
    pub fn new(right_ascension: f64, declination: f64) -> Self {
        let right_ascension = right_ascension % (std::f64::consts::PI*2.);
        let declination = declination.max(-90f64.to_radians()).min(90f64.to_radians()); // TODO: Is there a cleaner way to do this?
        Self {
            right_ascension,
            declination,
        }
    }

    pub fn from_hour_angle(hour_angle: HourAngle, declination: f64) -> Self {
        Self::new(hour_angle.to_radians(), declination)
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct Degree {
    pub base: i16,
    pub arc_mins: u8,
    pub arc_secs: f64
}


impl Degree {
    pub fn new(base: i16, arc_mins: u8, arc_secs: f64) -> Self {
        Self {
            base,
            arc_mins,
            arc_secs,
        }
    }

    pub fn to_f64(self) -> f64 {
        self.base as f64 + (self.arc_mins as f64)/60. + (self.arc_secs)/3600.
    }
}

#[derive(Default, Copy, Clone, Debug)]
pub struct HourAngle {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: f64
}

impl HourAngle {
    pub fn new(hours: u8, minutes: u8, seconds: f64) -> Self {
        Self {
            hours,
            minutes,
            seconds,
        }
    }
    pub fn to_sec(&self) -> f64 {
        (self.hours as u32 * 3600 + self.minutes as u32 * 60) as f64 + self.seconds
    }

    pub fn max_secs() -> f64 {
        (24 * 3600) as f64
    }

    pub fn to_radians(&self) -> f64 {
        self.to_sec() * std::f64::consts::PI / 43200f64
    }

    pub fn to_degrees(&self) -> f64 {
        self.to_sec() / 240.
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Record {
    #[serde(alias = "#")]
    id: usize,
    identifier: String,
    typ: String,
    #[serde(alias = "coord1 (ICRS,J2000/2000)")]
    coord1: Option<String>,
    #[serde(alias = "coord2 (FK5,J2000/2000)")]
    coord2: Option<String>,
    #[serde(alias = "coord3 (FK4,B1950/1950)")]
    coord3: Option<String>,
    #[serde(alias = "coord4 (Gal,J2000/2000)")]
    coord4: Option<String>,
    pm : Option<String>,
    plx: Option<f64>,
    radvel: Option<f64>,
    redshift: Option<f64>,
    cz: Option<f64>,
    #[serde(alias = "Mag U")]
    mag_u: Option<f64>,
    #[serde(alias = "Mag B")]
    mag_b: Option<f64>,
    #[serde(alias = "Mag V")]
    mag_v: Option<f64>,
    #[serde(alias = "Mag R")]
    mag_r: Option<f64>,
    #[serde(alias = "Mag I")]
    mag_i: Option<f64>,
    #[serde(alias = "spec. type")]
    spec_type: Option<String>,
    #[serde(alias = "morph. type")]
    morph_type: Option<String>,
    #[serde(alias = "ang. size")]
    ang_size: Option<String>,
    #[serde(alias = "pretty name")]
    pretty_name: Option<String>
}

#[derive(Clone, Debug)]
pub enum SimbadError {
    CoordNotFound,
    Unspecified
}

impl Display for SimbadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for SimbadError {}

pub fn import_records<P: AsRef<Path>>(path: P) -> Result<Vec<Record>, Box<dyn std::error::Error>> {
    let mut rdr = ReaderBuilder::new().delimiter(';' as u8).from_path(path)?;
    let mut records = vec![];
    for result in rdr.deserialize::<Record>() {
        if let Ok(record) = result {
            records.push(record);
        }
    }
    Ok(records)
}

pub fn import<P: AsRef<Path>>(path: P) -> Result<Vec<Star>, Box<dyn std::error::Error>> {
    let records = import_records(path)?;
    let mut stars = vec![];
    for record in records {
            if record.plx.is_none() { continue; }
            let dist = 1./(record.plx.ok_or(SimbadError::Unspecified)?/1000.)*3.26;
            let dist = if dist.is_finite() { dist } else { 0. };
            let coord1 = parse_coord(record.coord1.as_ref().ok_or(SimbadError::CoordNotFound)?);
            let coord2 = parse_coord(record.coord2.as_ref().ok_or(SimbadError::CoordNotFound)?);
            let coord3 = parse_coord(record.coord3.as_ref().ok_or(SimbadError::CoordNotFound)?);
            let coords = [coord1, coord2, coord3].into_iter().filter_map(|x| x).collect::<Vec<_>>();
            let coord = average_coord(&coords);
            let name = record.identifier;
            if record.id == 0 { println!("{:#?}", dist)}
            let pos = StellarPosition::new(dist, coord.right_ascension, coord.declination);
            if record.spec_type.is_none() { continue; }
            let spec_type = record.spec_type.unwrap();
            if name.ends_with("B") {continue;}
            let name = record.pretty_name.unwrap_or_default();
            let star = Star {
                id : record.id,
                pos : pos.into(),
                name,
                class: spec_type,
                constellation: "?".to_string(),
            };
            stars.push(star);
    }
    Ok(stars)
}
fn parse_coord(input: &str) -> Option<EquatorialCoordinate> {
    let splits = input.split_whitespace().collect::<Vec<_>>();
    if splits.len() < 6 { return None; }
    let ra = HourAngle::new(splits[0].parse::<u8>().ok()?, splits[1].parse::<u8>().ok()?, splits[2].parse::<f64>().ok()?);
    let ra = ra.to_radians();
    let dec = Degree::new((&splits[3][1..]).parse::<i16>().ok()?, splits[4].parse::<u8>().ok()?, splits[5].parse::<f64>().ok()?);
    let dec = dec.to_f64();
    let dec = (dec * if &splits[3][0..1] == "-" { -1. } else { 1. }).to_radians();
    Some(EquatorialCoordinate::new(ra, dec))
}

fn parse_coord4(input: &str) -> Option<EquatorialCoordinate> {
    let splits = input.split_whitespace().collect::<Vec<_>>();
    let ra = splits[0].parse::<f64>().ok()?.to_radians();
    let dec = splits[1].parse::<f64>().ok()?.to_radians();
    Some(EquatorialCoordinate::new(ra, dec))
}

fn average_coord(coords: &[EquatorialCoordinate]) -> EquatorialCoordinate {
    let ra = coords.iter().map(|x| x.right_ascension).sum::<f64>()/(coords.len() as f64);
    let dec = coords.iter().map(|x| x.declination).sum::<f64>()/(coords.len() as f64);
    EquatorialCoordinate::new(ra, dec)
}
