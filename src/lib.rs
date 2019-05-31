use nix::errno::Errno;
use nix::Result;
use nix::sys::socket;
use std::mem;
use std::os::unix::io::RawFd;


#[allow(non_camel_case_types)]
#[repr(C)]
pub struct sockaddr_vm {
    pub svm_family: libc::sa_family_t,
    pub svm_reserved1: u16,
    pub svm_port: i32,
    pub svm_cid: i32,
    pub svm_zero: [u8; 4],
}

unsafe fn sockaddr_vm(cid: i32, port: i32) -> (sockaddr_vm, libc::socklen_t) {
    let mut addr: sockaddr_vm = mem::zeroed();
    addr.svm_family = libc::AF_VSOCK as libc::sa_family_t;

    addr.svm_port = port;
    addr.svm_cid = cid;

    (addr, mem::size_of::<sockaddr_vm>() as libc::socklen_t)
}


pub struct Vsock {
    fd: RawFd
}

impl Vsock {
    pub fn new() -> Self {
        let socket_fd = socket::socket(socket::AddressFamily::Vsock,
                                       socket::SockType::Stream,
                                       socket::SockFlag::empty(),
                                       None).unwrap();
        Vsock { fd: socket_fd, }
    }

    pub fn connect(&self, cid: i32, port: i32) -> Result<()> {

        let res = unsafe {
            let (addr, len) = sockaddr_vm(cid, port);
            libc::connect(self.fd, mem::transmute(&addr), len)
        };

        return Errno::result(res).map(drop);
    }

    pub fn accept(&self) -> Result<Vsock> {
        let client_fd = socket::accept(self.fd)?;

        Ok(Vsock {fd: client_fd})
    }

    pub fn bind(&self, cid: i32, port: i32) -> Result<()> {
        let res = unsafe {
            let (addr, len) = sockaddr_vm(cid, port);
            libc::bind(self.fd, mem::transmute(&addr), len)
        };

        return Errno::result(res).map(drop);
    }

    pub fn getsockname(&self) -> Result<(i32, i32)> {
        let addr: sockaddr_vm;

        let res = unsafe {
            addr =  mem::zeroed();
            let mut addrlen: libc::socklen_t = mem::size_of::<sockaddr_vm>()
                                               as libc::socklen_t;
            libc::getsockname(self.fd, mem::transmute(&addr), &mut addrlen)
        };

        Errno::result(res)?;

        return Ok((addr.svm_cid, addr.svm_port));
    }

    pub fn listen(&self, backlog: usize) -> Result<()> {
        socket::listen(self.fd, backlog)
    }

    pub fn recv(&self, buf: &mut [u8], flags: socket::MsgFlags) -> Result<usize> {
        socket::recv(self.fd, buf, flags)
    }

    pub fn send(&self, buf: &[u8], flags: socket::MsgFlags) -> Result<usize> {
        socket::send(self.fd, buf, flags)
    }
}

impl Drop for Vsock {

    fn drop(&mut self) {
        let _ = nix::unistd::close(self.fd);
    }

}