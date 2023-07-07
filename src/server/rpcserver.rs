use std::collections::BTreeMap;
use std::net::SocketAddr;
//use std::ops::Range;
use std::sync::Arc;
use std::thread::JoinHandle;

//use sha2::Sha256;

use async_rdma::{QueuePairEndpoint, QueuePairEndpointBuilder, Gid, RdmaBuilder, LocalMrReadAccess, LocalMrWriteAccess};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status, transport::Server};

use crate::proto::rdma::{QpEndpointRequest, QpEndpoint};
use crate::proto::rdma::rdma_server::{Rdma, RdmaServer};
use crate::rdma_types::MemChunk;

#[derive(Debug)]
pub struct RdmaConnectionHandler {
    addr: SocketAddr,
//    rdma: Arc<Mutex<async_rdma::Rdma>>,
    qpe: QueuePairEndpoint,
}

impl RdmaConnectionHandler {
    pub fn new(peer: SocketAddr, peer_qpe: QueuePairEndpoint) -> Self {
        RdmaConnectionHandler { addr: peer, qpe: peer_qpe }
    }

    #[tokio::main]
    #[tracing::instrument]
    pub async fn run(&mut self) -> std::io::Result<()> {
        tracing::debug!("Start handler thread for: {:?}-{:}", self.qpe, self.addr);

        // Connect via QP Endpoint sent from remote side and use it to get an RDMA handle.
        let rdma = RdmaBuilder::default().ibv_connect(self.qpe).await?;

        tracing::debug!("Connected!: {:?}-{:}-{:?}", self.qpe, self.addr, rdma);
        // Do stuff with connection
        loop {
            let mut lmr = rdma.receive().await?;
            tracing::debug!("Got LocalMr: {:?}", lmr);
            
            // This is safe because as_mut_pointer returns a RwLockGuard.
            // This means when mc goes out of scope the lock is freed, we don't get a double free
            // when lmr goes out of scope.
            let mc = unsafe { *(*lmr.as_mut_ptr() as *mut MemChunk<u8, 4096>) };
            
            //let lmr: async_rdma::LocalMr = TryFrom::try_from(foo).unwrap();
            tracing::debug!("MC: {:?}", mc);
            
            rdma.send_local_mr(lmr).await?;

            mc.check();
            //let mem: Vec<u8> = foo.into();
            
        }
    }
}

pub struct RdmaService {
    rdma: Arc<Mutex<async_rdma::Rdma>>,
    handlers: Mutex<BTreeMap<(u32, u16), JoinHandle<Result<(), std::io::Error>>>>,
}

impl RdmaService {
    #[tokio::main]
    pub async fn run(rdma: Arc<Mutex<async_rdma::Rdma>>, addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
        let service = RdmaService { rdma, handlers: Mutex::new(BTreeMap::new())};

        Server::builder()
            .add_service(RdmaServer::new(service))
            .serve(addr)
            .await?;
        Ok(())
    }    
}

fn gid_from_vec_u8(v: Vec<u8>) -> Gid {
    let bytes: [u8; 16] = v.try_into().unwrap_or_else(|v: Vec<u8>| panic!("Bad Vec: {:?}", v));
    Gid::from_raw(bytes)
}

#[tonic::async_trait]
impl Rdma for RdmaService {
    async fn exchange(&self, req: Request<QpEndpoint>) -> Result<Response<QpEndpoint>, Status> {
        let remote = req.remote_addr().expect("No remote IP??");
        let req = req.into_inner();
        //let local_rdma = Arc::new(Mutex::new(RdmaBuilder::default().ibv_connect(remote)));
        let local_rdma = self.rdma.clone();
        let local_qpe = local_rdma.lock().await.get_qp_endpoint();
        let remote_gid = gid_from_vec_u8(req.gid);
        let remote_qpe = QueuePairEndpointBuilder::default()
                                                    .qp_num(req.num)
                                                    .lid(req.lid as u16)
                                                    .gid(remote_gid)
                                                    .build().unwrap();

        // Connect and then spawn thread to handle qp interactions.
        let mut handler = RdmaConnectionHandler::new(remote, remote_qpe);

        self.handlers.lock().await.insert((*local_qpe.qp_num(), *local_qpe.lid()), std::thread::spawn(move || { handler.run() } ));

        Ok(Response::new(QpEndpoint { num: *local_qpe.qp_num(), lid: *local_qpe.lid() as u32, gid: local_qpe.gid().as_raw().as_slice().into() }))
    }
}