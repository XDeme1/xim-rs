mod connection;

use xim_parser::{bstr::BString, CommitData, ErrorCode, ErrorFlag, InputStyle, Request};

pub use self::connection::{XimConnection, XimConnections};

#[derive(Debug, thiserror::Error)]
pub enum ServerError {
    #[error("Client doesn't exists")]
    ClientNotExists,
    #[error("Can't read xim message {0}")]
    ReadProtocol(#[from] xim_parser::ReadError),
    #[error("Client send error code: {0:?}, detail: {1}")]
    XimError(xim_parser::ErrorCode, BString),
    #[error("Invalid reply from client")]
    InvalidReply,
    #[error("Another instance is running")]
    AlreadyRunning,
    #[error(transparent)]
    Other(Box<dyn std::error::Error>),
}

pub trait ServerHandler<S: Server> {
    type InputContext;
    type InputStyleArray: AsRef<[InputStyle]>;

    fn input_styles(&self) -> Self::InputStyleArray;

    fn handle_connect(&mut self, server: &mut S) -> Result<(), ServerError>;

    fn handle_create_ic(
        &mut self,
        server: &mut S,
        connection: &mut XimConnection,
        input_method_id: u16,
        input_style: InputStyle,
    ) -> Result<(), ServerError>;
}

pub trait Server {
    fn error(
        &mut self,
        client_win: u32,
        code: ErrorCode,
        detail: BString,
        input_method_id: Option<u16>,
        input_context_id: Option<u16>,
    ) -> Result<(), ServerError>;

    fn commit(
        &mut self,
        client_win: u32,
        input_method_id: u16,
        input_context_id: u16,
        s: &str,
    ) -> Result<(), ServerError>;

    fn set_event_mask(
        &mut self,
        client_win: u32,
        input_method_id: u16,
        input_context_id: u16,
        forward_event_mask: u32,
        synchronous_event_mask: u32,
    ) -> Result<(), ServerError>;
}

impl<S: ServerCore> Server for S {
    fn error(
        &mut self,
        client_win: u32,
        code: ErrorCode,
        detail: BString,
        input_method_id: Option<u16>,
        input_context_id: Option<u16>,
    ) -> Result<(), ServerError> {
        let mut flag = ErrorFlag::empty();

        let input_method_id = if let Some(id) = input_method_id {
            flag |= ErrorFlag::INPUTMETHODIDVALID;
            id
        } else {
            0
        };

        let input_context_id = if let Some(id) = input_context_id {
            flag |= ErrorFlag::INPUTCONTEXTIDVALID;
            id
        } else {
            0
        };

        self.send_req(
            client_win,
            Request::Error {
                input_method_id,
                input_context_id,
                code,
                detail,
                flag,
            },
        )
    }

    fn commit(
        &mut self,
        client_win: u32,
        input_method_id: u16,
        input_context_id: u16,
        s: &str,
    ) -> Result<(), ServerError> {
        self.send_req(
            client_win,
            Request::Commit {
                input_method_id,
                input_context_id,
                data: CommitData::Chars {
                    commited: ctext::utf8_to_compound_text(s),
                    syncronous: false,
                },
            },
        )
    }

    fn set_event_mask(
        &mut self,
        client_win: u32,
        input_method_id: u16,
        input_context_id: u16,
        forward_event_mask: u32,
        synchronous_event_mask: u32,
    ) -> Result<(), ServerError> {
        self.send_req(
            client_win,
            Request::SetEventMask {
                input_method_id,
                input_context_id,
                forward_event_mask,
                synchronous_event_mask,
            },
        )
    }
}

pub trait ServerCore {
    fn send_req(&mut self, client_win: u32, req: Request) -> Result<(), ServerError>;
}
