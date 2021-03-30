
use crate::runner::Runner;
use crate::msg_handler::MsgHandler;
use crate::dev::Dev;

trait Filter
where Self: Runner + Dev + MsgHandler
{
}
