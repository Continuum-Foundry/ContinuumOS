#![no_std]
#![no_main]
mod runtime;
use runtime::*;
use core::{arch::{asm,global_asm},panic::PanicInfo};
global_asm!(r#"
.section .text.fixture_entry
.global fixture_entry
.type fixture_entry,@function
fixture_entry:
    mov rdi,rsi
    mov rsi,rdx
    sub rsp,8
    call {main}
    ud2
.size fixture_entry,.-fixture_entry
"#,main=sym main);
static mut SELF:u64=0;
static mut PARENT:u64=0;
fn me()->u64{unsafe{SELF}}
fn word(i:usize)->u64{unsafe{(0x70000000 as *const u64).add(i).read_volatile()}}
fn invoke(op:u64,a:u64,b:u64,c:u64,d:u64)->(u64,u64){
    let e;let v;unsafe{asm!("syscall",inlateout("rax")op=>e,in("rdi")a,in("rsi")b,
        inlateout("rdx")c=>v,in("r10")d,in("r8")0u64,in("r9")0u64,lateout("rcx")_,lateout("r11")_);}(e,v)
}
fn done(code:u64)->!{invoke(PROCESS_EXIT,code,word(3),0,0);unsafe{asm!("ud2",options(noreturn))}}
fn check(v:bool){if !v{log(b"supervisor-failed");done(99);}}
fn ok(op:u64,a:u64,b:u64,c:u64,d:u64)->u64{let(e,v)=invoke(op,a,b,c,d);check(e==0);v}
fn yield_now(){ok(THREAD_YIELD,word(4),0,0,0);}
fn now()->u64{ok(CLOCK_NOW,0,0,0,0)}
fn wait_ns(n:u64){let end=now()+n;while now()<end{yield_now();}}
fn encode<const N:usize>(words:[u64;N])->[u8;96]{
    let mut b=[0;96];for(i,w)in words.iter().enumerate(){b[i*8..i*8+8].copy_from_slice(&w.to_le_bytes());}b
}
fn context()->(u64,u64){let mut b=[0u8;32];ok(ABI_QUERY,b.as_mut_ptr() as u64,32,0,0);
    (u64::from_le_bytes(b[16..24].try_into().unwrap()),u64::from_le_bytes(b[24..32].try_into().unwrap()))}
fn request(mode:u64,units:u64)->[u64;12]{let(p,s)=context();[1,mode,p,s,0,1,1048576,units,1,0,0,0]}
fn create_request(w:[u64;12])->(u64,u64){let b=encode(w);invoke(PROCESS_CREATE,b.as_ptr() as u64,96,0,0)}
fn create(mode:u64,units:u64)->u64{let(e,v)=create_request(request(mode,units));check(e==0);v}
fn start(t:u64){ok(PROCESS_START,t,0,0,0);}
fn spawn(mode:u64,units:u64)->u64{let t=create(mode,units);start(t);t}
fn join(t:u64,expected:u64){
    loop{let(e,v)=invoke(PROCESS_JOIN,t,0,0,0);if e==WOULDBLOCK{yield_now();continue;}check(e==0&&v==expected);break;}
    check(invoke(PROCESS_TERMINATE,t,0,0,0).0==STALEEPOCH);
}
fn close(h:u64){ok(HANDLE_CLOSE,h,0,0,0);}
fn event()->u64{ok(EVENT_CREATE,word(3),0,0,0)}
fn park(){
    let h=event();let item=encode([h,0,0,0]);let req=encode([item.as_ptr() as u64,1,u64::MAX,0]);
    ok(WAIT_MANY,req.as_ptr() as u64,32,0,0);done(98);
}
fn pair()->[u64;2]{let mut out=[0u8;16];ok(CHANNEL_CREATE,word(3),out.as_mut_ptr() as u64,16,0);
    [u64::from_le_bytes(out[..8].try_into().unwrap()),u64::from_le_bytes(out[8..].try_into().unwrap())]}
fn memory()->u64{ok(MEMORY_CREATE,word(3),65536,32|64,0)}
fn send(h:u64,bytes:&[u8],gift:u64)->(u64,u64){
    let attach=encode([gift,7|16|32|64,0]); // move
    let req=encode([bytes.as_ptr() as u64,bytes.len() as u64,attach.as_ptr() as u64,if gift==0{0}else{1}]);
    invoke(CHANNEL_SEND,h,req.as_ptr() as u64,32,0)
}
fn quota_case(mode:u64){
    match mode {
        1100=>{ // live handle slots; denied duplicate cannot invalidate self
            let mut h=[0;10];for x in &mut h{*x=ok(HANDLE_DUPLICATE,word(3),7,0,0);}
            check(invoke(HANDLE_DUPLICATE,word(3),7,0,0).0==QUOTAEXCEEDED);
            for x in h{close(x);}ok(OBJECT_INFO,word(3),0,0,0);
        },
        1101=>{ // lifetime timer and object credits cannot be laundered by close
            for _ in 0..4{let h=ok(TIMER_CREATE,word(3),0,0,0);close(h);}
            check(invoke(TIMER_CREATE,word(3),0,0,0).0==QUOTAEXCEEDED);
            for _ in 0..6{let h=event();close(h);}
            check(invoke(EVENT_CREATE,word(3),0,0,0).0==QUOTAEXCEEDED);
        },
        1102=>{ // bounded pair, aggregate bytes/messages, atomic transfer denial
            let [tx,_rx]=pair();let mut out=[0u8;16];
            check(invoke(CHANNEL_CREATE,word(3),out.as_mut_ptr() as u64,16,0).0==QUOTAEXCEEDED);
            let gift=memory();let payload=[17u8;4096];
            check(send(tx,&payload,gift).0==0);check(invoke(OBJECT_INFO,gift,0,0,0).0==BADHANDLE);
            check(send(tx,&payload,0).0==0);
            let retained=event();check(send(tx,b"x",retained).0==QUOTAEXCEEDED);
            ok(OBJECT_INFO,retained,0,0,0);
            check(send(tx,b"",0).0==0);check(send(tx,b"",0).0==0);
            check(send(tx,b"",0).0==QUOTAEXCEEDED);
        },
        1103=>{ // cumulative allocation byte ceiling, even after actual free
            let mut count=0;
            loop{let(e,p)=invoke(MEMORY_ALLOCATE,65536,4096,0,0);
                if e!=0{check(e==QUOTAEXCEEDED&&count>8&&count<16);break;}
                unsafe{check((p as *const u64).read_volatile()==0);}
                ok(MEMORY_FREE,p,0,0,0);count+=1;}
            check(invoke(MEMORY_ALLOCATE,65537,4096,0,0).0==QUOTAEXCEEDED);
        },
        1104=>{ // mapping ceiling; last memory handle closes with live maps
            let h=memory();let mut count=0;
            loop{let desc=encode([0x60000000+count*65536,0,65536,0]);
                let(e,_)=invoke(MEMORY_MAP,word(3),h,desc.as_ptr() as u64,32);
                if e!=0{check(e==QUOTAEXCEEDED&&count>0&&count<16);break;}count+=1;}
            close(h);
        },
        _=>done(97)
    }
    log(b"quota-healthy");done(0);
}
fn parent(mode:u64){
    let child=spawn(if mode==1200{1000}else{1006},1);
    check(create_request(request(1000,1)).0==QUOTAEXCEEDED);
    if mode==1200{join(child,0);done(0);}
    yield_now(); // child parks before parent fault
    unsafe{asm!("ud2",options(noreturn));}
}
fn root(wave:u64){
    // Explicit authority, inheritance, threads and priority escalation probes.
    for(index,value,expected)in[(2,202,ACCESSDENIED),(3,999,ACCESSDENIED),(4,1,ACCESSDENIED),
        (5,8,ACCESSDENIED),(6,2097152,ACCESSDENIED),(8,2,QUOTAEXCEEDED),
        (9,1,ACCESSDENIED),(10,1,ACCESSDENIED),(0,99,INVALIDARGUMENT)]{
        let mut w=request(1000,1);w[index]=value;check(create_request(w).0==expected);
    }
    match wave {
        0=>{
            for(mode,result)in[(1000,0),(1001,262),(1002,1),(1003,2)]{let t=spawn(mode,1);join(t,result);}
            let t=create(1000,1);check(ok(PROCESS_STATE,t,0,0,0)==0);
            ok(PROCESS_TERMINATE,t,0,0,0);join(t,768);
            let hog=spawn(1005,1);let healthy=spawn(1000,1);join(healthy,0);
            wait_ns(30_000_000);ok(PROCESS_TERMINATE,hog,0,0,0);join(hog,768);
            check(create_request(request(1000,1)).0==QUOTAEXCEEDED);
        },
        1=>{
            for mode in 1100..=1104{let t=spawn(mode,1);join(t,0);}
            let p=spawn(1200,2);join(p,0);
            check(create_request(request(1000,1)).0==QUOTAEXCEEDED);
            for _ in 0..4{check(invoke(DIAGNOSTIC_LOG,b"rate".as_ptr() as u64,4,0,0).0==0);}
            check(invoke(DIAGNOSTIC_LOG,b"rate".as_ptr() as u64,4,0,0).0==QUOTAEXCEEDED);
        },
        2=>{
            let t=spawn(1001,1);join(t,262);
            for _ in 0..2{wait_ns(30_000_000);ok(SUPERVISOR_RESTART,0,0,0,0);let t=spawn(1001,1);join(t,262);}
            check(invoke(SUPERVISOR_RESTART,0,0,0,0).0==QUOTAEXCEEDED);
            let p=spawn(1201,2);
            let orphan=loop{let t=ok(PROCESS_FIRST_CHILD,p,0,0,0);if t!=0{break t;}yield_now();};
            join(orphan,768);join(p,262);
            let blocked=spawn(1006,1);yield_now();ok(PROCESS_TERMINATE,blocked,0,0,0);join(blocked,768);
            let healthy=spawn(1000,1);join(healthy,0);
        },
        3=>{
            spawn(1006,1);spawn(1007,1);spawn(1005,1);create(1000,1);
            for _ in 0..4{yield_now();}
            log(b"root-failure-ready");unsafe{asm!("ud2",options(noreturn));}
        },
        _=>done(96)
    }
    done(0);
}
extern "C" fn main(token:u64,parent_token:u64){
    unsafe{SELF=token;}
    unsafe{PARENT=parent_token;}
    let mode=word(0);
    if (500..504).contains(&mode){root(mode-500);}
    match mode{
        1000=>{check(ok(PROCESS_STATE,me(),0,0,0)==1);
            for op in [PROCESS_START,PROCESS_TERMINATE,PROCESS_JOIN,PROCESS_STATE] {
                check(invoke(op,unsafe{PARENT},0,0,0).0==ACCESSDENIED);
            }
            check(invoke(SUPERVISOR_RESTART,0,0,0,0).0==ACCESSDENIED);
            log(b"healthy-progress");done(0);},
        1001=>unsafe{asm!("ud2",options(noreturn))},
        1002=>panic!("worker panic"),
        1003=>{check(invoke(MEMORY_ALLOCATE,u64::MAX,4096,0,0).0==QUOTAEXCEEDED);done(2);},
        1005=>loop{unsafe{asm!("pause");}},
        1006=>park(),
        1007=>{let [tx,_]=pair();let h=memory();check(send(tx,b"held",h).0==0);park();},
        1100..=1104=>quota_case(mode),
        1200|1201=>parent(mode),
        _=>done(95)
    }
}
#[panic_handler]
fn panic(_: &PanicInfo)->!{log(b"panic");done(1)}
