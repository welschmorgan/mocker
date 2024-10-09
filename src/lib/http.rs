use std::{
  collections::VecDeque,
  io::Write,
  ops::{Deref, DerefMut},
  str::FromStr,
};

use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{Error, ErrorKind};

#[derive(
  Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, EnumIter, Hash,
)]
#[repr(C)]
pub enum Method {
  #[serde(rename = "POST")]
  Post,
  #[serde(rename = "GET")]
  Get,
  #[serde(rename = "PUT")]
  Put,
  #[serde(rename = "PATCH")]
  Patch,
  #[serde(rename = "DELETE")]
  Delete,
  #[serde(rename = "HEAD")]
  Head,
  #[serde(rename = "OPTIONS")]
  Options,
}

#[derive(Copy, Clone, Debug, EnumIter)]
pub enum Status {
  Continue,
  SwitchingProtocols,
  Processing,
  EarlyHints,

  OK,
  Created,
  Accepted,
  NonAuthoritativeInformation,
  NoContent,
  ResetContent,
  PartialContent,
  MultiStatus,
  AlreadyReported,
  ContentDifferent,
  IMUsed,

  MultipleChoices,
  MovedPermanently,
  Found,
  SeeOther,
  NotModified,
  UseProxy,
  Unused,
  TemporaryRedirect,
  PermanentRedirect,
  TooManyRedirects,

  BadRequest,
  Unauthorized,
  PaymentRequired,
  Forbidden,
  NotFound,
  MethodNotAllowed,
  NotAcceptable,
  ProxyAuthenticationRequired,
  RequestTimeOut,
  Conflict,
  Gone,
  LengthRequired,
  PreconditionFailed,
  RequestEntityTooLarge,
  RequestURITooLong,
  UnsupportedMediaType,
  RequestedRangeUnsatisfiable,
  ExpectationFailed,
  ImATeapot,
  PageExpired,
  BadMappingOrMisdirectedRequest,
  UnprocessableEntity,
  Locked,
  MethodFailure,
  TooEarly,
  UpgradeRequired,
  InvalidDigitalSignature,
  PreconditionRequired,
  TooManyRequests,
  RequestHeaderFieldsTooLarge,
  RetryWith,
  BlockedByWindowsParentalControls,
  UnavailableForLegalReasons,
  UnrecoverableError,

  NoResponse,
  SSLCertificateError,
  SSLCertificateRequired,
  HTTPRequestSentToHTTPSPort,
  TokenExpiredOrInvalid,
  ClientClosedRequest,

  InternalServerError,
  NotImplemented,
  BadGatewayOuProxyError,
  ServiceUnavailable,
  GatewayTimeOut,
  HTTPVersionNotSupported,
  VariantAlsoNegotiates,
  InsufficientStorage,
  LoopDetected,
  BandwidthLimitExceeded,
  NotExtended,
  NetworkAuthenticationRequired,
  UnknownError,
  WebServerIsDown,
  ConnectionTimedOut,
  OriginIsUnreachable,
  ATimeoutOccurred,
  SSLHandshakeFailed,
  InvalidSSLCertificate,
  RailgunError,
}

impl TryFrom<u16> for Status {
  type Error = crate::Error;

  fn try_from(value: u16) -> crate::Result<Self> {
    for status in Status::iter() {
      if status.descr().0 == value {
        return Ok(status);
      }
    }
    Err(Error::new(
      ErrorKind::Parse,
      Some(format!("not a http status: {}", value)),
      None,
    ))
  }
}

impl Status {
  pub fn code(&self) -> u16 {
    self.descr().0
  }

  pub fn text(&self) -> &'static str {
    self.descr().1
  }

  pub fn details(&self) -> &'static str {
    self.descr().2
  }

  pub fn descr(&self) -> (u16, &'static str, &'static str) {
    match self {
      Self::Continue => (100, "Continue", "	Attente de la suite de la requête."),
      Self::SwitchingProtocols => (101, "Switching Protocols", "	Acceptation du changement de protocole."),
      Self::Processing => (102, "Processing", "WebDAV RFC 25185,6	Traitement en cours (évite que le client dépasse le temps d’attente limite)."),
      Self::EarlyHints => (103, "Early Hints", "RFC 82977	(Expérimental) Dans l'attente de la réponse définitive, le serveur renvoie des liens que le client peut commencer à télécharger."),

      Self::OK => (200, "OK", "RFC 19458	Requête traitée avec succès. La réponse dépendra de la méthode de requête utilisée."),
      Self::Created => (201, "Created", "RFC 19458	Requête traitée avec succès et création d’un document."),
      Self::Accepted => (202, "Accepted", "RFC 19458	Requête traitée, mais sans garantie de résultat."),
      Self::NonAuthoritativeInformation => (203, "Non-Authoritative Information", "	Information renvoyée, mais générée par une source non certifiée."),
      Self::NoContent => (204, "No Content", "RFC 19458	Requête traitée avec succès mais pas d’information à renvoyer."),
      Self::ResetContent => (205, "Reset Content", "RFC 20689	Requête traitée avec succès, la page courante peut être effacée."),
      Self::PartialContent => (206, "Partial Content", "RFC 20689	Une partie seulement de la ressource a été transmise."),
      Self::MultiStatus => (207, "Multi-Status", "WebDAV	Réponse multiple."),
      Self::AlreadyReported => (208, "Already Reported", "WebDAV	Le document a été envoyé précédemment dans cette collection."),
      Self::ContentDifferent => (210, "Content Different", "WebDAV	La copie de la ressource côté client diffère de celle du serveur (contenu ou propriétés)."),
      Self::IMUsed => (226, "IM Used", "RFC 322910	Le serveur a accompli la requête pour la ressource, et la réponse est une représentation du résultat d'une ou plusieurs manipulations d'instances appliquées à l'instance actuelle."),

      Self::MultipleChoices => (300, "Multiple Choices", "RFC 19458	L’URI demandée se rapporte à plusieurs ressources."),
      Self::MovedPermanently => (301, "Moved Permanently", "RFC 19458	Document déplacé de façon permanente."),
      Self::Found => (302, "Found", "RFC 19458	Document déplacé de façon temporaire."),
      Self::SeeOther => (303, "See Other", "RFC 20689	La réponse à cette requête est ailleurs."),
      Self::NotModified => (304, "Not Modified", "RFC 19458	Document non modifié depuis la dernière requête."),
      Self::UseProxy => (305, "Use Proxy (depuis HTTP/1.1)", "RFC 20689	La requête doit être ré-adressée au proxy."),
      Self::Unused => (306, "(inutilisé)", "RFC 261611	La RFC 261611 indique que ce code inutilisé est réservé, car il était utilisé dans une ancienne version de la spécification. Il signifiait « Les requêtes suivantes doivent utiliser le proxy spécifié »12."),
      Self::TemporaryRedirect => (307, "Temporary Redirect", "	La requête doit être redirigée temporairement vers l’URI spécifiée sans changement de méthode13."),
      Self::PermanentRedirect => (308, "Permanent Redirect", "	La requête doit être redirigée définitivement vers l’URI spécifiée sans changement de méthode14."),
      Self::TooManyRedirects => (310, "Too many Redirects", "	La requête doit être redirigée de trop nombreuses fois, ou est victime d’une boucle de redirection."),

      Self::BadRequest => (400, "Bad Request", "RFC 19458	La syntaxe de la requête est erronée."),
      Self::Unauthorized => (401, "Unauthorized", "RFC 19458	Une authentification est nécessaire pour accéder à la ressource."),
      Self::PaymentRequired => (402, "Payment Required", "RFC 20689	Paiement requis pour accéder à la ressource."),
      Self::Forbidden => (403, "Forbidden", "RFC 19458	Le serveur a compris la requête, mais refuse de l'exécuter. Contrairement à l'erreur 401, s'authentifier ne fera aucune différence. Sur les serveurs où l'authentification est requise, cela signifie généralement que l'authentification a été acceptée mais que les droits d'accès ne permettent pas au client d'accéder à la ressource."),
      Self::NotFound => (404, "Not Found", "RFC 19458	Ressource non trouvée."),
      Self::MethodNotAllowed => (405, "Method Not Allowed", "RFC 20689	Méthode de requête non autorisée."),
      Self::NotAcceptable => (406, "Not Acceptable", "RFC 20689	La ressource demandée n'est pas disponible dans un format qui respecterait les en-têtes « Accept » de la requête."),
      Self::ProxyAuthenticationRequired => (407, "Proxy Authentication Required", "RFC 20689	Accès à la ressource autorisé par identification avec le proxy."),
      Self::RequestTimeOut => (408, "Request Time-out", "RFC 20689	Temps d’attente d’une requête du client, écoulé côté serveur. D'après les spécifications HTTP : « Le client n'a pas produit de requête dans le délai que le serveur était prêt à attendre. Le client PEUT répéter la demande sans modifications à tout moment ultérieur »15."),
      Self::Conflict => (409, "Conflict", "RFC 20689	La requête ne peut être traitée à la suite d'un conflit avec l'état actuel du serveur."),
      Self::Gone => (410, "Gone", "RFC 20689	La ressource n'est plus disponible et aucune adresse de redirection n’est connue."),
      Self::LengthRequired => (411, "Length Required", "RFC 20689	La longueur de la requête n’a pas été précisée."),
      Self::PreconditionFailed => (412, "Precondition Failed", "RFC 20689	Préconditions envoyées par la requête non vérifiées."),
      Self::RequestEntityTooLarge => (413, "Request Entity Too Large", "RFC 20689	Traitement abandonné dû à une requête trop importante."),
      Self::RequestURITooLong => (414, "Request-URI Too Long", "RFC 20689	URI trop longue."),
      Self::UnsupportedMediaType => (415, "Unsupported Media Type", "RFC 20689	Format de requête non supporté pour une méthode et une ressource données."),
      Self::RequestedRangeUnsatisfiable => (416, "Requested range unsatisfiable", "	Champs d’en-tête de requête « range » incorrect."),
      Self::ExpectationFailed => (417, "Expectation failed", "	Comportement attendu et défini dans l’en-tête de la requête insatisfaisante."),
      Self::ImATeapot => (418, "I’m a teapot", "RFC 232416	« Je suis une théière » : Ce code est défini dans la RFC 232417 datée du 1er avril 1998, Hyper Text Coffee Pot Control Protocol."),
      Self::PageExpired => (419, "Page expired", "	Ressource expirée"),
      Self::BadMappingOrMisdirectedRequest => (421, "Bad mapping / Misdirected Request", "	La requête a été envoyée à un serveur qui n'est pas capable de produire une réponse (par exemple, car une connexion a été réutilisée)."),
      Self::UnprocessableEntity => (422, "Unprocessable entity", "WebDAV	L’entité fournie avec la requête est incompréhensible ou incomplète."),
      Self::Locked => (423, "Locked", "WebDAV	L’opération ne peut avoir lieu car la ressource est verrouillée."),
      Self::MethodFailure => (424, "Method failure", "WebDAV	Une méthode de la transaction a échoué."),
      Self::TooEarly => (425, "Too Early", "RFC 847018	Le serveur ne peut traiter la demande car elle risque d'être rejouée."),
      Self::UpgradeRequired => (426, "Upgrade Required", "RFC 281719	Le client devrait changer de protocole, par exemple au profit de TLS/1.0."),
      Self::InvalidDigitalSignature => (427, "Invalid digital signature", "Microsoft	La signature numérique du document est non-valide."),
      Self::PreconditionRequired => (428, "Precondition Required", "RFC 658520	La requête doit être conditionnelle."),
      Self::TooManyRequests => (429, "Too Many Requests", "RFC 658520	Le client a émis trop de requêtes dans un délai donné."),
      Self::RequestHeaderFieldsTooLarge => (431, "Request Header Fields Too Large", "RFC 658520	Les entêtes HTTP émises dépassent la taille maximale admise par le serveur."),
      Self::RetryWith => (449, "Retry With", "Microsoft	La requête devrait être renvoyée après avoir effectué une action."),
      Self::BlockedByWindowsParentalControls => (450, "Blocked by Windows Parental Controls", "Microsoft	Cette erreur est produite lorsque les outils de contrôle parental de Microsoft Windows sont activés et bloquent l’accès à la page."),
      Self::UnavailableForLegalReasons => (451, "Unavailable For Legal Reasons", "RFC 772521	La ressource demandée est inaccessible pour des raisons d'ordre légal."),
      Self::UnrecoverableError => (456, "Unrecoverable Error", "WebDAV Erreur irrécupérable."),
      Self::NoResponse => (444, "No Response", "Nginx	Indique que le serveur n'a retourné aucune information vers le client et a fermé la connexion."),
      Self::SSLCertificateError => (495, "SSL Certificate Error", "Nginx	Une extension de l'erreur 400 Bad Request, utilisée lorsque le client a fourni un certificat invalide."),
      Self::SSLCertificateRequired => (496, "SSL Certificate Required", "Nginx	Une extension de l'erreur 400 Bad Request, utilisée lorsqu'un certificat client requis n'est pas fourni."),
      Self::HTTPRequestSentToHTTPSPort => (497, "HTTP Request Sent to HTTPS Port", "Nginx	Une extension de l'erreur 400 Bad Request, utilisée lorsque le client envoie une requête HTTP vers le port 443 normalement destiné aux requêtes HTTPS."),
      Self::TokenExpiredOrInvalid => (498, "Token expired/invalid", "Nginx	Le jeton a expiré ou est invalide."),
      Self::ClientClosedRequest => (499, "Client Closed Request", "Nginx	Le client a fermé la connexion avant de recevoir la réponse. Cette erreur se produit quand le traitement est trop long côté serveur22."),

      Self::InternalServerError => (500, "Internal Server Error", "RFC 19458	Erreur interne du serveur."),
      Self::NotImplemented => (501, "Not Implemented", "RFC 19458	Fonctionnalité réclamée non supportée par le serveur."),
      Self::BadGatewayOuProxyError => (502, "Bad Gateway ou Proxy Error", "RFC 19458	En agissant en tant que serveur proxy ou passerelle, le serveur a reçu une réponse invalide depuis le serveur distant."),
      Self::ServiceUnavailable => (503, "Service Unavailable", "RFC 19458	Service temporairement indisponible ou en maintenance."),
      Self::GatewayTimeOut => (504, "Gateway Time-out", "RFC 20689	Temps d’attente d’une réponse d’un serveur à un serveur intermédiaire écoulé."),
      Self::HTTPVersionNotSupported => (505, "HTTP Version not supported", "RFC 20689	Version HTTP non gérée par le serveur."),
      Self::VariantAlsoNegotiates => (506, "Variant Also Negotiates", "RFC 229523	Erreur de négociation. Transparent content negociation."),
      Self::InsufficientStorage => (507, "Insufficient storage", "WebDAV	Espace insuffisant pour modifier les propriétés ou construire la collection."),
      Self::LoopDetected => (508, "Loop detected", "WebDAV	Boucle dans une mise en relation de ressources (RFC 584224)."),
      Self::BandwidthLimitExceeded => (509, "Bandwidth Limit Exceeded", "	Utilisé par de nombreux serveurs pour indiquer un dépassement de quota."),
      Self::NotExtended => (510, "Not extended", "RFC 277425	La requête ne respecte pas la politique d'accès aux ressources HTTP étendues."),
      Self::NetworkAuthenticationRequired => (511, "Network authentication required", "RFC 658520	Le client doit s'authentifier pour accéder au réseau. Utilisé par les portails captifs pour rediriger les clients vers la page d'authentification."),

      Self::UnknownError => (520, "Unknown Error", "Cloudflare	Réponse générique lorsque le serveur d'origine retourne un résultat imprévu."),
      Self::WebServerIsDown => (521, "Web Server Is Down", "Cloudflare	Le serveur a refusé la connexion depuis Cloudflare."),
      Self::ConnectionTimedOut => (522, "Connection Timed Out", "Cloudflare	Cloudflare n'a pas eu de retour avec le serveur d'origine dans les temps."),
      Self::OriginIsUnreachable => (523, "Origin Is Unreachable", "Cloudflare	Cloudflare n'a pas réussi à joindre le serveur d'origine. Cela peut se produire en cas d'échec de résolution de nom de serveur DNS."),
      Self::ATimeoutOccurred => (524, "A Timeout Occurred", "Cloudflare	Cloudflare a établi une connexion TCP avec le serveur d'origine mais n'a pas reçu de réponse HTTP avant l'expiration du délai de connexion."),
      Self::SSLHandshakeFailed => (525, "SSL Handshake Failed", "Cloudflare	Cloudflare n'a pas pu négocier un SSL/TLS handshake avec le serveur d'origine."),
      Self::InvalidSSLCertificate => (526, "Invalid SSL Certificate", "Cloudflare	Cloudflare n'a pas pu valider le certificat SSL présenté par le serveur d'origine."),
      Self::RailgunError => (527, "Railgun Error", "Cloudflare	La requête a dépassé le délai de connexion ou a échoué après que la connexion WAN a été établie."),
    }
  }
}

impl Method {
  pub fn repr(&self) -> String {
    format!("{:?}", self).to_uppercase()
  }
}

impl FromStr for Method {
  type Err = crate::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    for meth in Method::iter() {
      if format!("{:?}", meth).eq_ignore_ascii_case(s) {
        return Ok(meth);
      }
    }
    Err(Error::new(
      ErrorKind::Parse,
      Some(format!("Unknown http method '{}'", s)),
      None,
    ))
  }
}

impl Display for Method {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.repr())
  }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, EnumIter)]
pub enum Version {
  V1_0,
  V1_1,
  V2,
}

impl Version {
  pub fn repr(&self) -> &'static str {
    match self {
      Self::V1_0 => "HTTP/1.0",
      Self::V1_1 => "HTTP/1.1",
      Self::V2 => "HTTP/2",
    }
  }
}

impl FromStr for Version {
  type Err = crate::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    for vers in Version::iter() {
      if vers.repr().eq_ignore_ascii_case(s) {
        return Ok(vers);
      }
    }
    Err(Error::new(
      ErrorKind::Parse,
      Some(format!("Unknown http method '{}'", s)),
      None,
    ))
  }
}

impl Display for Version {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "{}", self.repr())
  }
}

impl Default for Version {
  fn default() -> Self {
    Self::V1_1
  }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RequestStart {
  pub method: Method,
  pub target: String,
  pub version: Version,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResponseStart {
  pub version: Version,
  pub status: u16,
  pub reason: Option<String>,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum StartLine {
  Request(RequestStart),
  Response(ResponseStart),
}

impl StartLine {
  pub fn request<M: Into<Method>, T: AsRef<str>, V: Into<Version>>(m: M, t: T, v: V) -> Self {
    return Self::Request(RequestStart {
      method: m.into(),
      target: t.as_ref().to_string(),
      version: v.into(),
    });
  }

  pub fn response<V: Into<Version>, R: Into<Option<String>>>(v: V, s: u16, r: R) -> Self {
    let reason: Option<String> = r.into();
    return Self::Response(ResponseStart {
      version: v.into(),
      status: s,
      reason: reason.or_else(|| {
        if let Ok(status) = Status::try_from(s) {
          return Some(status.descr().1.to_string());
        }
        None
      }),
    });
  }

  pub fn as_request(&self) -> Option<&RequestStart> {
    match self {
      Self::Request(v) => Some(v),
      _ => None,
    }
  }

  pub fn as_response(&self) -> Option<&ResponseStart> {
    match self {
      Self::Response(v) => Some(v),
      _ => None,
    }
  }

  pub fn as_request_mut(&mut self) -> Option<&mut RequestStart> {
    match self {
      Self::Request(v) => Some(v),
      _ => None,
    }
  }

  pub fn as_response_mut(&mut self) -> Option<&mut ResponseStart> {
    match self {
      Self::Response(v) => Some(v),
      _ => None,
    }
  }
}

impl FromStr for StartLine {
  type Err = crate::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let parts = s.split(' ').collect::<Vec<_>>();
    if parts.len() < 2 {
      return Err(Error::new(
        ErrorKind::IO,
        Some(format!(
          "invalid http start line, expected >= {} parts but got {}",
          2,
          parts.len()
        )),
        None,
      ));
    }
    if parts[0].starts_with("HTTP") {
      // is status line (response)
      Ok(StartLine::response(
        parts[0].parse::<Version>()?,
        parts[1].parse::<u16>()?,
        parts.get(2).map(|v| v.to_string()),
      ))
    } else {
      // is request line
      Ok(StartLine::request(
        parts[0].parse::<Method>()?,
        parts[1].to_string(),
        parts
          .get(2)
          .ok_or_else(|| {
            Error::new(
              ErrorKind::Parse,
              Some(format!(
                "Invalid http start line, missing version in '{}'",
                s
              )),
              None,
            )
          })?
          .parse::<Version>()?,
      ))
    }
  }
}
use std::fmt::Display;
impl Display for StartLine {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Request(start) => format!("{} {} {}", start.method, start.target, start.version),
        Self::Response(start) => format!(
          "{} {}{}",
          start.version,
          start.status,
          match &start.reason {
            Some(r) => format!(" {}", r),
            None => String::new(),
          }
        ),
      }
    )
  }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Buffer {
  start_line: StartLine,
  headers: Vec<(String, String)>,
  body: Vec<u8>,
}

unsafe impl Send for Buffer {}
unsafe impl Sync for Buffer {}

impl Default for Buffer {
  fn default() -> Self {
    Self {
      start_line: StartLine::response(Version::default(), 200u16, None),
      headers: Default::default(),
      body: Default::default(),
    }
  }
}
impl Buffer {
  pub fn with_start_line(mut self, v: StartLine) -> Self {
    self.start_line = v;
    self
  }

  pub fn with_headers<K: AsRef<str>, V: AsRef<str>, I: IntoIterator<Item = (K, V)>>(
    mut self,
    v: I,
  ) -> Self {
    self.headers = v
      .into_iter()
      .map(|(k, v)| (k.as_ref().to_string(), v.as_ref().to_string()))
      .collect::<Vec<_>>();
    self
  }

  pub fn with_header<K: AsRef<str>, V: AsRef<str>>(mut self, k: K, v: V) -> Self {
    self
      .headers
      .push((k.as_ref().to_string(), v.as_ref().to_string()));
    self
  }

  pub fn with_body<B: AsRef<str>>(mut self, v: B) -> Self {
    self.body.clear();
    self.append_body(v);
    self
  }

  pub fn append_body<B: AsRef<str>>(&mut self, v: B) {
    let data = v.as_ref().bytes().collect::<Vec<_>>();
    self.body.extend_from_slice(&data);
    self.set_header("Content-Length", self.body.len().to_string());
  }

  pub fn set_header<K: AsRef<str>, V: AsRef<str>>(&mut self, k: K, v: V) {
    match self
      .headers
      .iter_mut()
      .find(|(hk, _hv)| hk.eq_ignore_ascii_case(k.as_ref()))
    {
      Some((_hk, hv)) => *hv = v.as_ref().to_string(),
      None => self
        .headers
        .push((k.as_ref().to_string(), v.as_ref().to_string())),
    }
  }

  pub fn start_line(&self) -> &StartLine {
    &self.start_line
  }

  pub fn start_line_mut(&mut self) -> &mut StartLine {
    &mut self.start_line
  }

  pub fn header<K: AsRef<str>>(&self, uk: K) -> Option<&String> {
    self.headers.iter().find_map(|(k, v)| {
      if k.eq_ignore_ascii_case(uk.as_ref()) {
        return Some(v);
      }
      None
    })
  }

  pub fn headers(&self) -> &Vec<(String, String)> {
    &self.headers
  }

  pub fn body(&self) -> &Vec<u8> {
    &self.body
  }

  pub fn write_to<W: Write>(&self, mut w: W) -> crate::Result<()> {
    writeln!(w, "{}", self.start_line)?;
    for (key, value) in self.headers() {
      writeln!(w, "{}: {}", key, value)?;
    }
    if !self.body.is_empty() {
      writeln!(w)?;
      w.write(&self.body())?;
    }
    Ok(())
  }
}

impl Display for Buffer {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    let mut buf = vec![];
    self.write_to(&mut buf).map_err(|_| std::fmt::Error)?;
    let s = std::str::from_utf8(&buf).map_err(|_| std::fmt::Error)?;
    write!(f, "{}", s)
  }
}

impl FromStr for Buffer {
  type Err = crate::Error;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    let mut lines = s.lines().collect::<VecDeque<_>>();
    let start_line = lines.remove(0).ok_or_else(|| {
      Error::new(
        ErrorKind::Parse,
        Some(format!("invalid http buffer, missing start line:\n{}", s)),
        None,
      )
    })?;
    let start_line = start_line.parse()?;
    let mut body_mode = false;
    let mut headers = vec![];
    let mut body = vec![];
    for line in lines {
      if line.is_empty() {
        body_mode = true;
      } else {
        if body_mode {
          body.push(line);
        } else {
          headers.push(line);
        }
      }
    }
    let headers = headers
      .iter()
      .map(|header| {
        header.split_once(':').ok_or_else(|| {
          Error::new(
            ErrorKind::Parse,
            Some(format!("invalid header '{}'", header)),
            None,
          )
        })
      })
      .collect::<Vec<_>>();
    for kv in &headers {
      if kv.is_err() {
        return Err(kv.as_ref().err().unwrap().clone());
      }
    }
    let headers = headers
      .iter()
      .filter_map(|h| {
        if h.is_ok() {
          let kv = h.as_ref().ok().unwrap();
          return Some((kv.0, kv.1.trim()));
        }
        None
      })
      .collect::<Vec<_>>();
    let body = body.join("\n");
    Ok(
      Self::default()
        .with_start_line(start_line)
        .with_headers(headers)
        .with_body(body),
    )
  }
}

#[cfg(test)]
mod tests {
  use crate::Method;

  use super::{Buffer, StartLine, Version};

  #[test]
  fn response() {
    let buf = Buffer::default()
      .with_start_line(StartLine::response(
        Version::V1_0,
        200 as u16,
        Some("OK".to_string()),
      ))
      .with_headers([("Content-Type", "application/json")])
      .with_body("test");
    let buf = buf.to_string();
    assert_eq!(
      buf.as_str(),
      r#"HTTP/1.0 200 OK
Content-Type: application/json
Content-Length: 4

test"#
    );
  }

  #[test]
  fn request() {
    let buf = Buffer::default()
      .with_start_line(StartLine::request(Method::Get, "/", Version::V1_0))
      .with_headers([("Content-Type", "application/json")])
      .with_body("test");
    let buf = buf.to_string();
    assert_eq!(
      buf.as_str(),
      r#"GET / HTTP/1.0
Content-Type: application/json
Content-Length: 4

test"#
    );
  }
}
