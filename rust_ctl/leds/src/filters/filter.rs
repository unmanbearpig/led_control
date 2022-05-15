use crate::dev::Dev;
use crate::msg_handler::MsgHandler;
use crate::runner::Runner;

trait Filter
where
    Self: Runner + Dev + MsgHandler,
{
}
