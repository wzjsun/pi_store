// use std::sync::{Arc, RwLock};
// use std::string::String;
// use std::vec::Vec;
// use std::usize;
// use std::clone::Clone;
// use std::ops::Deref;
// use std::cell::RefCell;
// use std::boxed::FnOnce;
// use std::rc::Rc;
// use std::mem::replace;

// use fnv::FnvHashMap;
// use rocksdb::*;
// use rocksdb;

// use pi_lib::sinfo::{EnumType};
// use pi_lib::atom::{Atom};
// use pi_lib::guid::{Guid};
// use pi_lib::bon::{ReadBuffer, WriteBuffer, Encode, Decode};
// use pi_base::task::TaskType;
// use pi_base::pi_base_impl::STORE_TASK_POOL;
// use pi_db::db::{Bin, TabKV, SResult, DBResult, IterResult, KeyIterResult, NextResult, TxCallback, TxQueryCallback, Txn, TabTxn, MetaTxn, Tab, OpenTab, Ware, WareSnapshot, Filter, TxState, Iter, CommitResult, RwLog, TabMeta};
// use pi_db::tabs::{TabLog, Tabs};

// /*
// * db异步访问任务类型
// */
// const ASYNC_DB_TYPE: TaskType = TaskType::Sync;

// /*
// * db异步访问任务优先级
// */
// const DB_PRIORITY: u64 = 20;

// const TIMEOUT: usize = 100;

// const ROOT: &str = "_$rocksdb";
// const SINFO: &str = "_$sinfo";

// /*
// * 信息
// */
// lazy_static! {
// 	pub static ref DB_ASYNC_FILE_INFO: Atom = Atom::from("DB asyn file");
// }

// //对应接口的Tab 创建事务
// pub struct FTab{
//     pub tab: rocksdb::DB,
// 	pub name: Atom,
//     pub write_opts: Arc<WriteOptions>,
//     pub read_opts: Arc<ReadOptions>,
// }

// #[derive(Clone)]
// pub struct FileTab(Arc<FTab>);

// impl Tab for FileTab {
//     fn new(path: &Atom) -> Self {
//         let opts = get_default_options();
//         let write_opts = WriteOptions::default();
//         let read_opts = ReadOptions::default();
//         let mut wopts = WriteOptions::default();
//         wopts.set_sync(false);                                                                      //无需同步
//         wopts.disable_wal(false);                                                                    //需要写前导日志

//         let db = match rocksdb::DB::open(&opts, path.deref()) {
//             Err(_e) => {
//                 match rocksdb::DB::repair(opts, path.deref()) {
//                     Err(e) => {
//                         panic!("!!!!!!db repair failed, err: {}", e.to_string());
//                     },
//                     Ok(_) => {
//                         let opts = get_default_options();
//                         match rocksdb::DB::open(&opts, path.deref()){
//                             Err(e) => {
//                                 panic!("!!!!!!db repair ok, but can not open, err: {}", e.to_string());
//                             },
//                             Ok(v) => {
//                                 v
//                             },
//                         }
//                     },
//                 }
//             },
//             Ok(v) => {
//                 v
//             },
//         };

// 		FileTab(Arc::new(
//             FTab{
//                 tab: db,
//                 name: path.clone(),
//                 write_opts: Arc::new(write_opts),
//                 read_opts: Arc::new(read_opts),
//             }
//         ))
// 	}

//     fn transaction(&self, id: &Guid, writable: bool) -> Arc<TabTxn> {
// 		let tab = &self.0;
// 		Arc::new(FileTabTxn::new(FTabTxn{
//             tab: self.clone(),
//             id: id.clone(),
//             batch: WriteBatch::default(),
//             _writable: writable,
//             rwlog: FnvHashMap::default(),
//             state: TxState::Ok,
//             write_opts: tab.write_opts.clone(),
//             read_opts: tab.read_opts.clone(),
//         }))
//     }
// }

// pub struct FTabTxn{
//     pub tab: FileTab,
//     pub batch: WriteBatch,
//     pub id: Guid,
// 	pub _writable: bool,
// 	pub rwlog: FnvHashMap<Bin, RwLog>,
// 	pub state: TxState,
//     pub write_opts: Arc<WriteOptions>,
//     pub read_opts: Arc<ReadOptions>,
// }

// #[derive(Clone)]
// pub struct FileTabTxn(Rc<RefCell<FTabTxn>>);

// impl FileTabTxn{
//     pub fn new(tab_txn: FTabTxn) -> Self{
//         FileTabTxn(Rc::new(RefCell::new(tab_txn)))
//     }
// }

// impl Txn for FileTabTxn{
//     // 获得事务的状态
// 	fn get_state(&self) -> TxState{
//         self.0.borrow().state.clone()
//     }
// 	// 预提交一个事务
// 	fn prepare(&self, _timeout:usize, _cb: TxCallback) -> DBResult{
//         self.0.borrow_mut().state = TxState::Preparing;
//         Some(Ok(()))
//     }
// 	// 提交一个事务
// 	fn commit(&self, cb: TxCallback) -> CommitResult{
//         let sclone = self.0.clone();
//         sclone.borrow_mut().state = TxState::Committing;
//         send_task(Box::new(move || {
//             let batch = replace(&mut sclone.borrow_mut().batch, WriteBatch::default());
//             let mut sclone = sclone.borrow_mut();
//             match sclone.tab.0.tab.write_opt(batch, &sclone.write_opts) {
//                 Err(e) => {
//                     sclone.state = TxState::CommitFail;
//                     free(sclone);
//                     cb(Err(e.to_string()))
//                 },
//                 Ok(_) => {
//                     sclone.state = TxState::Commited;
//                     free(sclone);
//                     cb(Ok(()))
//                 },
//             }
//         }));
//         None
//     }
// 	// 回滚一个事务
// 	fn rollback(&self, _cb: TxCallback) -> DBResult{
//         let sclone = self.0.clone();
//         sclone.borrow_mut().state = TxState::Rollbacking;
//         Some(Ok(()))
//     }
// }

// impl TabTxn for FileTabTxn{
//     // 键锁，key可以不存在，根据lock_time的值决定是锁还是解锁
// 	fn key_lock(&self, _arr: Arc<Vec<TabKV>>, _lock_time: usize, _readonly: bool, _cb: TxCallback) -> DBResult {
// 		None
// 	}
// 	// 查询
// 	fn query(&self,arr: Arc<Vec<TabKV>>,_lock_time: Option<usize>,_readonly: bool, cb: TxQueryCallback,) -> Option<SResult<Vec<TabKV>>> {
//         let sclone = self.0.clone();
//         let func = move || {
//             let mut value_arr = Vec::new();
//             let sclone = sclone.borrow();
//             for tabkv in arr.iter() {
//                 let mut value = None;
//                 match sclone.tab.0.tab.get(tabkv.key.as_slice()) {
//                     Ok(None) => (),
//                     Ok(v) => {
//                         value = Some(Arc::new(Vec::from(v.unwrap().deref())));
//                         ()
//                     },
//                     Err(e) => {
//                         free(sclone);
//                         cb(Err(e.to_string()));
//                         return;
//                     },
//                 }
//                 value_arr.push(
//                     TabKV{
//                         ware:tabkv.ware.clone(),
//                         tab: tabkv.tab.clone(),
//                         key: tabkv.key.clone(),
//                         index: tabkv.index,
//                         value: value,
//                     }
//                 )
//             }
//             free(sclone);
//             cb(Ok(value_arr))

//         };
//         send_task(Box::new(func));
//         None
// 	}
// 	// 修改，插入、删除及更新
// 	fn modify(&self, arr: Arc<Vec<TabKV>>, _lock_time: Option<usize>, _readonly: bool, cb: TxCallback) -> DBResult {
// 		let sclone = self.0.clone();
//         let func = move || {
//             let mut sclone = sclone.borrow_mut();
//             for tabkv in arr.iter() {
//                 if tabkv.value == None {
//                     match sclone.tab.0.tab.delete(&tabkv.key.as_slice()) {
//                         Ok(_) => (),
//                         Err(e) =>{
//                             free(sclone);
//                             cb(Err(e.to_string()));
//                             return;
//                         },
//                     };
//                 } else {
//                     match sclone.batch.put(&tabkv.key.as_slice(), &tabkv.value.clone().unwrap().as_slice()) {
//                         Ok(_) => (),
//                         Err(e) => {
//                             free(sclone);
//                             cb(Err(e.to_string()));
//                             return;
//                         },
//                     };
//                 }
//             }
//             free(sclone);
//             cb(Ok(()))
//         };
//         send_task(Box::new(func));
//         None
//     }
// 	// 迭代
// 	fn iter(&self,key: Option<Bin>,descending: bool,filter: Filter, cb: Arc<Fn(IterResult)>,) -> Option<IterResult> {
//         let sclone = self.0.clone();
//         let func = move || {
//             let mut rocksdb_iter = sclone.borrow().tab.0.tab.raw_iterator();
//             if key == None {
//                 if descending {
//                     rocksdb_iter.seek_to_last();
//                 } else {
//                     rocksdb_iter.seek_to_first();
//                 }
//             } else {
//                 if descending {
//                     rocksdb_iter.seek_for_prev(key.unwrap().as_slice());
//                 } else {
//                     rocksdb_iter.seek(key.unwrap().as_slice());
//                 }
//             }
//             cb(Ok(Box::new(FDBIterator::new(rocksdb_iter, descending, filter))))
//         };
//         send_task(Box::new(func));
//         None
// 	}
// 	// 迭代
// 	fn key_iter(&self, key: Option<Bin>,descending: bool,filter: Filter, cb: Arc<Fn(KeyIterResult)>,) -> Option<KeyIterResult> {
// 		let sclone = self.0.clone();
//         let func = move || {
//             let mut rocksdb_iter = sclone.borrow().tab.0.tab.raw_iterator();
//             if key == None {
//                 if descending {
//                     rocksdb_iter.seek_to_last();
//                 } else {
//                     rocksdb_iter.seek_to_first();
//                 }
//             } else {
//                 if descending {
//                     rocksdb_iter.seek_for_prev(key.unwrap().as_slice());
//                 } else {
//                     rocksdb_iter.seek(key.unwrap().as_slice());
//                 }
//             }
//             cb(Ok(Box::new(KeyFDBIterator::new(rocksdb_iter, descending, filter))))
//         };
//         send_task(Box::new(func));
//         None
// 	}
// 	// 索引迭代
// 	fn index(&self,_tab: &Atom,_index_key: &Atom,_key: Option<Bin>,_descending: bool,_filter: Filter,_cb: Arc<Fn(IterResult)>,) -> Option<IterResult> {
// 		None
// 	}
// 	// 表的大小
// 	fn tab_size(&self, _cb: Arc<Fn(Result<usize, String>)>) -> Option<Result<usize, String>> {
// 		None
// 	}
// }

// #[derive(Clone)]
// pub struct FileMetaTxn(Arc<TabTxn>);

// impl FileMetaTxn{
//     //tab_txn 必须是Arc<FileTabTxn>
//     fn new(tab_txn: Arc<TabTxn>) -> FileMetaTxn{
//         FileMetaTxn(tab_txn)
//     }
// }

// impl MetaTxn for FileMetaTxn {
// 	// 创建表、修改指定表的元数据
// 	fn alter(&self, tab: &Atom, meta: Option<Arc<TabMeta>>, cb: TxCallback) -> DBResult{
//         let mut key = WriteBuffer::new();
//         tab.encode(&mut key);
//         let key = Arc::new(key.unwrap());

//         let value = match meta {
//             Some(v) => {
//                 let mut value = WriteBuffer::new();
//                 v.encode(&mut value);
//                 Some(Arc::new(value.unwrap()))
//             },
//             None => None,
//         };

//         let tabkv = TabKV{
//             ware: Atom::from(""),
//             tab: Atom::from(""),
//             key: key,
//             index: 0,
//             value: value,
//         };
//         self.0.modify(Arc::new(vec![tabkv]), None, false, cb)
// 	}

// 	// 快照拷贝表
// 	fn snapshot(&self, _tab: &Atom, _from: &Atom, _cb: TxCallback) -> DBResult{
// 		Some(Ok(()))
// 	}
// 	// 修改指定表的名字
// 	fn rename(&self, _tab: &Atom, _new_name: &Atom, _cb: TxCallback) -> DBResult {
// 		Some(Ok(()))
// 	}
// }
// impl Txn for FileMetaTxn {
// 	// 获得事务的状态
// 	fn get_state(&self) -> TxState {
//         self.0.get_state()
// 	}
// 	// 预提交一个事务
// 	fn prepare(&self, timeout: usize, cb: TxCallback) -> DBResult {
// 		self.0.prepare(timeout, cb)
// 	}
// 	// 提交一个事务
// 	fn commit(&self, cb: TxCallback) -> CommitResult {
// 		self.0.commit(cb)
// 	}
// 	// 回滚一个事务
// 	fn rollback(&self, cb: TxCallback) -> DBResult {
// 		self.0.rollback(cb)
// 	}
// }

// #[derive(Clone)]
// pub struct DB{
//     name: Atom,
//     tabs: Arc<RwLock<Tabs<FileTab>>>
// }

// impl DB {
//     //create FileDB, return OK(DB) or Err(String) if open db with IO Error
// 	pub fn new(name: Atom) -> Result<Self, String>{
//         let root = String::from(ROOT) + "/" + name.as_str() + "/"; //根路径 + 库名
//         let sinfo_path = root.clone() + SINFO;
//         let opts = get_default_options();
//         let db = match rocksdb::DB::open(&opts, &sinfo_path) {
//             Err(_e) => {
//                 match rocksdb::DB::repair(opts, &sinfo_path) {
//                     Err(e) => {
//                         panic!("!!!!!!db repair failed, err: {}", e.to_string());
//                     },
//                     Ok(_) => {
//                         let opts = get_default_options();
//                         match rocksdb::DB::open(&opts, &sinfo_path){
//                             Err(e) => {
//                                 panic!("!!!!!!db repair ok, but can not open, err: {}", e.to_string());
//                             },
//                             Ok(v) => {
//                                 v
//                             },
//                         }
//                     },
//                 }
//             },
//             Ok(v) => {
//                 v
//             },
//         };

//         let mut it = db.raw_iterator();
//         it.seek_to_first();
//         let mut tabs: Tabs<FileTab> = Tabs::new();
//         while it.valid() {
//             let v = it.value().unwrap();
//             tabs.set_tab_meta(Atom::decode(&mut ReadBuffer::new(&it.key().unwrap(), 0)).expect(""), Arc::new(TabMeta::decode(&mut ReadBuffer::new(&v, 0)).expect("")));
//             it.next();
//         }

//         tabs.set_tab_meta(Atom::from(SINFO), Arc::new(TabMeta::new(EnumType::Str, EnumType::Bool))); //添加元信息表的元信息
//         let a = Arc::new(RwLock::new(tabs));
// 		Ok(DB{
//             name: name,
//             tabs: a.clone()
//         })
// 	}
// }

// impl OpenTab for DB {
// 	// 打开指定的表，表必须有meta
// 	fn open<'a, T: Tab>(&self, tab: &Atom, _cb: Box<Fn(SResult<T>) + 'a>) -> Option<SResult<T>> {
//         let name = String::from(ROOT) + "/" + &self.name + "/" + tab;
// 		Some(Ok(T::new(&Atom::from(name))))
// 	}
// }

// impl Ware for DB {
// 	// 拷贝全部的表
// 	fn tabs_clone(&self) -> Arc<Ware> {
// 	    Arc::new(DB{
//             name: self.name.clone(),
//             tabs:Arc::new(RwLock::new(self.tabs.read().unwrap().clone_map()))
//         })
// 	}
// 	// 列出全部的表
// 	fn list(&self) -> Box<Iterator<Item=Atom>> {
// 		Box::new(self.tabs.read().unwrap().list())
// 	}
// 	// 获取该库对预提交后的处理超时时间, 事务会用最大超时时间来预提交
// 	fn timeout(&self) -> usize {
// 		TIMEOUT
// 	}
// 	// 表的元信息
// 	fn tab_info(&self, tab_name: &Atom) -> Option<Arc<TabMeta>> {
// 		self.tabs.read().unwrap().get(tab_name)
// 	}
// 	// 获取当前表结构快照
// 	fn snapshot(&self) -> Arc<WareSnapshot> {
// 		Arc::new(DBSnapshot(self.clone(), RefCell::new(self.tabs.read().unwrap().snapshot())))
// 	}
// }

// // 内存库快照
// pub struct DBSnapshot(DB, RefCell<TabLog<FileTab>>);

// impl WareSnapshot for DBSnapshot {
// 	// 列出全部的表
// 	fn list(&self) -> Box<Iterator<Item=Atom>> {
// 		Box::new(self.1.borrow().list())
// 	}
// 	// 表的元信息
// 	fn tab_info(&self, tab_name: &Atom) -> Option<Arc<TabMeta>> {
// 		self.1.borrow().get(tab_name)
// 	}
// 	// 检查该表是否可以创建
// 	fn check(&self, _tab: &Atom, _meta: &Option<Arc<TabMeta>>) -> SResult<()> {
// 		Ok(())
// 	}
// 	// 新增 修改 删除 表
// 	fn alter(&self, tab_name: &Atom, meta: Option<Arc<TabMeta>>) {
// 		self.1.borrow_mut().alter(tab_name, meta)
// 	}
// 	// 创建指定表的表事务
// 	fn tab_txn(&self, tab_name: &Atom, id: &Guid, writable: bool, cb: Box<Fn(SResult<Arc<TabTxn>>)>) -> Option<SResult<Arc<TabTxn>>> {
// 		self.1.borrow().build(&self.0, tab_name, id, writable, cb)
// 	}
// 	// 创建一个meta事务
// 	fn meta_txn(&self, id: &Guid) -> Arc<MetaTxn> {
//         Arc::new(FileMetaTxn::new(self.tab_txn(&Atom::from(SINFO), id, true, Box::new(|_r|{})).unwrap().expect("meta_txn")))
// 	}
// 	// 元信息预提交
// 	fn prepare(&self, id: &Guid) -> SResult<()>{
// 		(self.0).tabs.write().unwrap().prepare(id, &mut self.1.borrow_mut())
// 	}
// 	// 元信息提交
// 	fn commit(&self, id: &Guid){
// 		(self.0).tabs.write().unwrap().commit(id)
// 	}
// 	// 回滚
// 	fn rollback(&self, id: &Guid){
// 		(self.0).tabs.write().unwrap().rollback(id)
// 	}

// }

// pub struct FDBIterator{
//     it: Arc<RefCell<DBRawIterator>>,
//     descending: bool,
//     _filter: Filter,
// }

// impl FDBIterator{
//     pub fn new(it: DBRawIterator, descending: bool, filter: Filter) -> Self{
//         FDBIterator{
//             it: Arc::new(RefCell::new(it)),
//             descending:descending,
//             _filter: filter
//         }
//     }
// }

// impl Iter for FDBIterator{
//     type Item = (Bin, Bin);
// 	fn next(&mut self, cb: Arc<Fn(NextResult<Self::Item>)>) -> Option<NextResult<Self::Item>>{
//         let it = self.it.clone();
//         let descending = self.descending;
//         let func = move || {
//             let mut it = it.borrow_mut();
//             match it.valid(){
//                 true => {
//                     cb(Ok(Some((Arc::new(it.key().unwrap()), Arc::new(it.value().unwrap())))));
//                     match descending {
//                         true => it.prev(),
//                         false => it.next(),
//                     };
//                 },
//                 false => cb(Ok(None))
//             }
//         };
//         send_task(Box::new(func));
//         None
//     }
// }

// pub struct KeyFDBIterator{
//     it: DBRawIterator,
//     descending: bool,
//     _filter: Filter,
// }

// impl KeyFDBIterator{
//     pub fn new(it: DBRawIterator, descending: bool, filter: Filter) -> Self{
//         KeyFDBIterator{
//             it: it,
//             descending:descending,
//             _filter: filter
//         }
//     }
// }

// impl Iter for KeyFDBIterator{
//     type Item = Bin;
// 	fn next(&mut self, cb: Arc<Fn(NextResult<Self::Item>)>) -> Option<NextResult<Self::Item>>{
//         let mut it = self.it.clone();
//         let descending = self.descending;
//         let func = move || {
//             match it.valid(){
//                 true => {
//                     cb(Ok(Some(Arc::new(it.key().unwrap()))));
//                     match descending {
//                         true => it.prev(),
//                         false => it.next(),
//                     };
//                 },
//                 false => cb(Ok(None))
//             }
//         };
//         send_task(Box::new(func));
//         None
//     }
// }

// fn send_task(func: Box<FnOnce()>){
//     let &(ref lock, ref cvar) = &**STORE_TASK_POOL;
//     let mut task_pool = lock.lock().unwrap();
//     (*task_pool).push(ASYNC_DB_TYPE, DB_PRIORITY, func, DB_ASYNC_FILE_INFO.clone());
//     cvar.notify_one();
// }

// fn free<T>(_:T) {}

// fn get_default_options () -> Options {
//     let mut opts = Options::default();
//     //基础配置
//     opts.create_if_missing(true);                                                               //库不存在，则创建
//     opts.set_max_open_files(4096);                                                              //最大可以打开文件数，-1表示无限制，注意应小于linux当前shell配置的最大可以打开文件数
//     opts.set_max_background_flushes(2);                                                         //设置后台最大刷新线程数
//     opts.enable_statistics();                                                                   //允许统计数据库信息
//     opts.set_stats_dump_period_sec(300);                                                        //每5分钟将统计信息写入日志文件
//     //压缩配置
//     // opts.set_compression_type(DBCompressionType::Snappy);                                       //使用Snappy进行压缩
//     // opts.set_compression_per_level(&[
//     //     DBCompressionType::None,
//     //     DBCompressionType::None,
//     //     DBCompressionType::Snappy,
//     //     DBCompressionType::Snappy,
//     //     DBCompressionType::Snappy
//     // ]);                                                                                         //设置每级压缩，低级不压缩，高级使用Snappy压缩
//     // opts.set_compaction_readahead_size(2 * 1024 * 1024);                                        //压缩预读大小，HDD应该不小于2MB，以保证尽量顺序访问磁盘，SSD可以为0
//     opts.set_compaction_style(DBCompactionStyle::Universal);                                    //压缩样式
//     opts.set_max_background_compactions(2);                                                     //设置后台最大压缩线程数
//     //写相关配置
//     opts.set_use_fsync(true);                                                                   //设置落地时使用fsync还是fdatasync，设置为false将会提高落地效率，但在ext3中最好设置为true，以防止丢失数据
//     opts.set_allow_concurrent_memtable_write(false);                                            //设置是否允许并发写Memtable，一般关闭，因为当前兼容性不好
//     opts.set_write_buffer_size(0x2000000);                                                        //写缓冲大小，可以提高写性能，但会降低库打开性能，可在运行时改变
//     // opts.set_bytes_per_sync(1024 * 1024);                                                       //限制同步速度，这会在后台用异步线程将内存数据同步到文件
//     //文件配置
//     opts.set_max_bytes_for_level_base(256 * 1024 * 1024);                                        //设置L1的大小
//     opts.set_max_bytes_for_level_multiplier(1.0);                                               //设置multiplier
//     //WAL配置
//     // opts.set_wal_dir("./wal");                                                                  //设置wal的路径，默认和数据库同路径
//     opts.set_wal_recovery_mode(DBRecoveryMode::AbsoluteConsistency);                            //通过wal恢复的模式

//     let mut bopts = BlockBasedOptions::default();
//     bopts.set_block_size(16 * 1024 * 1024);                                                     //设置块大小
//     bopts.set_lru_cache(1024 * 1024);                                                           //设置块缓存大小
//     bopts.set_bloom_filter(10, false);                                                          //使用不基于块的Bloom过滤器
//     bopts.set_cache_index_and_filter_blocks(false);                                             //不缓存索引和过滤器块
//     opts
// }

// #[cfg(test)]
// use std::thread;
// #[cfg(test)]
// use std::time::Duration;
// #[cfg(test)]
// use pi_base::worker_pool::WorkerPool;
// #[cfg(test)]
// use pi_lib::sinfo::StructInfo;

// #[test]
// fn test(){
//     let worker_pool0 = Box::new(WorkerPool::new(3, 1024 * 1024, 1000));
//     worker_pool0.run(STORE_TASK_POOL.clone());

//     let tab_name = Atom::from("player");
//     let ware_name = Atom::from("file_test");
//     let db = DB::new(ware_name.clone()).expect("new db fail");
//     let snapshot = db.snapshot();
//     let guid = Guid(0);

//     let meta_txn = snapshot.meta_txn(&guid);

//     let sinfo = Arc::new(TabMeta::new(EnumType::Str, EnumType::Struct(Arc::new(StructInfo::new(tab_name.clone(), 8888)))));
//     snapshot.alter(&tab_name, Some(sinfo.clone()));

//     let tab_txn1 = snapshot.tab_txn(&Atom::from(SINFO), &guid, true, Box::new(|_r|{})).unwrap().expect("create player tab_txn fail");
//     let key1 = Arc::new(Vec::from(String::from("key1").as_bytes()));
//     let value1 = Arc::new(Vec::from(String::from("value1").as_bytes()));
//     let item1 = create_tabkv(ware_name.clone(), Atom::from(SINFO), key1.clone(), 0, Some(value1.clone()));
//     let arr =  Arc::new(vec![item1.clone()]);
//    // &tab_name, Some(sinfo.clone())
//     tab_txn1.modify(arr.clone(), None, false, Arc::new(move |alter|{
//         assert!(alter.is_ok());  //插入元信息成功

//         let meta_txn_clone = meta_txn.clone();
//         let meta_txn = meta_txn.clone();
//         meta_txn_clone.prepare(1000, Arc::new(move |prepare|{
//             assert!(prepare.is_ok());  //预提交元信息成功
//             meta_txn.commit(Arc::new(move |commit|{
//                 match commit {
//                     Ok(_) => (),//提交元信息成功
//                     Err(e) => panic!("{:?}", e),
//                 };
//                 //println!("meta_txn commit success");
//             }));
//         }));
//         //println!("assert is success");
//     }));

//     thread::sleep(Duration::from_millis(1000));
//     let key1 = Arc::new(Vec::from(String::from("key1").as_bytes()));
//     let value1 = Arc::new(Vec::from(String::from("value1").as_bytes()));
//     let key2 = Arc::new(Vec::from(String::from("key2").as_bytes()));
//     let value2 = Arc::new(Vec::from(String::from("value2").as_bytes()));
//     let key3 = Arc::new(Vec::from(String::from("key3").as_bytes()));
//     let value3 = Arc::new(Vec::from(String::from("value3").as_bytes()));

//     let item1 = create_tabkv(ware_name.clone(), tab_name.clone(), key1.clone(), 0, Some(value1.clone()));
//     let item2 = create_tabkv(ware_name.clone(), tab_name.clone(), key2.clone(), 0, Some(value2.clone()));
//     let item3 = create_tabkv(ware_name.clone(), tab_name.clone(), key3.clone(), 0, Some(value3.clone()));
//     let arr3 =  Arc::new(vec![item1.clone(), item2.clone(), item2.clone()]);

//     let tab_txn1 = snapshot.tab_txn(&tab_name, &guid, true, Box::new(|_r|{})).unwrap().expect("create player tab_txn fail");
//     let tab_txn2 = snapshot.tab_txn(&tab_name, &guid, true, Box::new(|_r|{})).unwrap().expect("create player tab_txn fail");
//     let tab_txn = snapshot.tab_txn(&tab_name, &guid, true, Box::new(|_r|{})).unwrap().expect("create player tab_txn fail");

//     //事务1插入key1, key2
//     let arr = Arc::new(vec![item1.clone(), item2.clone()]);
//     let tab_txn1_clone = tab_txn1.clone();
//     tab_txn1_clone.modify(arr.clone(), None, false, Arc::new(move |modify|{
//         match modify {
//             Ok(_) => (),//插入数据成功
//             Err(e) => panic!("{:?}", e),
//         };
//         println!("tab_txn1 insert key1, key2 is success");

//         //事务2插入key1
//         let item1 = item1.clone();
//         let item3 = item3.clone();
//         let tab_txn2 = tab_txn2.clone();
//         let tab_txn1 = tab_txn1.clone();
//         let tab_txn2_clone = tab_txn2.clone();
//         let arr3 = arr3.clone();
//         let tab_txn = tab_txn.clone();
//         let arr = Arc::new(vec![item1.clone()]);
//         tab_txn2_clone.modify(arr.clone(), None, false, Arc::new(move|modify|{
//             assert!(modify.is_ok());//插入数据成功
//             //println!("tab_txn2 insert key1 is fail");

//             //事务2插入key3
//             let tab_txn2_clone = tab_txn2.clone();
//             let tab_txn2 = tab_txn2.clone();
//             let tab_txn1 = tab_txn1.clone();
//             let arr3 = arr3.clone();
//             let tab_txn = tab_txn.clone();
//             let arr = Arc::new(vec![item3.clone()]);
//             tab_txn2_clone.modify(arr.clone(), None, false, Arc::new(move |modify|{
//                 match modify {
//                     Ok(_) => (),//插入数据成功
//                     Err(e) => panic!("{:?}", e),
//                 };
//                 //println!("tab_txn2 insert key3 is success");

//                 let tab_txn1_clone = tab_txn1.clone();
//                 let tab_txn1 = tab_txn1.clone();
//                 let tab_txn2 = tab_txn2.clone();
//                 let arr3 = arr3.clone();
//                 let tab_txn = tab_txn.clone();
//                 tab_txn1_clone.prepare(1000, Arc::new(move |prepare|{
//                     assert!(prepare.is_ok());  //事务1预提交成功
//                     //println!("tab_txn1 prepare is success");

//                     let tab_txn1 = tab_txn1.clone();
//                     let tab_txn2 = tab_txn2.clone();
//                     let tab_txn2_clone = tab_txn2.clone();
//                     let arr3 = arr3.clone();
//                     let tab_txn = tab_txn.clone();
//                     tab_txn2_clone.prepare(1000, Arc::new(move |prepare|{
//                         assert!(prepare.is_ok());  //事务2预提交成功
//                         //println!("tab_txn2 prepare is success");

//                         let tab_txn1 = tab_txn1.clone();
//                         let tab_txn2 = tab_txn2.clone();
//                         let arr3 = arr3.clone();
//                         let tab_txn = tab_txn.clone();
//                         tab_txn1.commit(Arc::new(move |commit|{
//                             assert!(commit.is_ok());  //事务1提交成功
//                             //println!("tab_txn1 commit is success");

//                             let tab_txn2 = tab_txn2.clone();
//                             let arr3 = arr3.clone();
//                             let tab_txn = tab_txn.clone();
//                             tab_txn2.commit(Arc::new(move |commit|{
//                                 assert!(commit.is_ok());  //事务2提交成功
//                                 //println!("tab_txn2 commit is success");

//                                 tab_txn.query(arr3.clone(), None, false, Arc::new(move |query|{
//                                     assert!(query.is_ok());  //查询数据成功
//                                     // let r = query.expect("");
//                                     // for v in r.iter(){
//                                     //     println!("-----------------------{}", String::from_utf8_lossy(v.value.as_ref().unwrap().as_slice()));
//                                     // }
//                                 }));
//                             }));
//                         }));
//                     }));
//                 }));
//             }));
//         }));
//     }));

//     thread::sleep(Duration::from_millis(3000));
// }

// #[cfg(test)]
// fn create_tabkv(ware: Atom,tab: Atom,key: Bin,index: usize,value: Option<Bin>,) -> TabKV{
//     TabKV{ware, tab, key, index, value}
// }
