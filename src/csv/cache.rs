use lru::LruCache;
use std::num::NonZeroUsize;
use crate::csv::CsvRecord;

/// 页面缓存
/// 使用LRU（最近最少使用）策略缓存最近访问的页面
pub struct PageCache {
    cache: LruCache<usize, Vec<CsvRecord<'static>>>,
}

impl PageCache {
    /// 创建新的页面缓存
    /// 
    /// # 参数
    /// - `capacity`: 缓存容量（最多缓存多少个页面）
    pub fn new(capacity: usize) -> Self {
        let capacity = NonZeroUsize::new(capacity.max(1)).unwrap();
        Self {
            cache: LruCache::new(capacity),
        }
    }

    /// 获取缓存的页面
    /// 
    /// # 参数
    /// - `page`: 页码
    /// 
    /// # 返回
    /// 如果缓存中存在该页面，返回Some，否则返回None
    pub fn get(&mut self, page: &usize) -> Option<&Vec<CsvRecord<'static>>> {
        self.cache.get(page)
    }

    /// 将页面放入缓存
    /// 
    /// # 参数
    /// - `page`: 页码
    /// - `records`: 该页的记录数据
    pub fn put(&mut self, page: usize, records: Vec<CsvRecord<'static>>) {
        self.cache.put(page, records);
    }

    /// 清空缓存
    pub fn clear(&mut self) {
        self.cache.clear();
    }

    /// 获取当前缓存大小
    pub fn len(&self) -> usize {
        self.cache.len()
    }

    /// 检查缓存是否为空
    pub fn is_empty(&self) -> bool {
        self.cache.len() == 0
    }
}

impl Default for PageCache {
    fn default() -> Self {
        Self::new(10) // 默认缓存10页
    }
}

