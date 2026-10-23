#include "/tmp/builduser/esp32-c6-zephyr-rust-dd73-97hQMW/zephyr-sys/wrapper.h"

// Static wrappers

size_t sys_count_bits__extern(const void *value, size_t len) { return sys_count_bits(value, len); }
uint32_t sys_gcd_u__extern(uint32_t a, uint32_t b) { return sys_gcd_u(a, b); }
uint32_t sys_gcd_s__extern(int32_t a, int32_t b) { return sys_gcd_s(a, b); }
uint64_t sys_lcm_u__extern(uint32_t a, uint32_t b) { return sys_lcm_u(a, b); }
uint64_t sys_lcm_s__extern(int32_t a, int32_t b) { return sys_lcm_s(a, b); }
bool k_is_user_context__extern(void) { return k_is_user_context(); }
void sys_dlist_init__extern(sys_dlist_t *list) { sys_dlist_init(list); }
void sys_dnode_init__extern(sys_dnode_t *node) { sys_dnode_init(node); }
bool sys_dnode_is_linked__extern(const sys_dnode_t *node) { return sys_dnode_is_linked(node); }
bool sys_dlist_is_head__extern(const sys_dlist_t *list, const sys_dnode_t *node) { return sys_dlist_is_head(list, node); }
bool sys_dlist_is_tail__extern(const sys_dlist_t *list, const sys_dnode_t *node) { return sys_dlist_is_tail(list, node); }
bool sys_dlist_is_empty__extern(const sys_dlist_t *list) { return sys_dlist_is_empty(list); }
bool sys_dlist_has_multiple_nodes__extern(const sys_dlist_t *list) { return sys_dlist_has_multiple_nodes(list); }
sys_dnode_t * sys_dlist_peek_head__extern(const sys_dlist_t *list) { return sys_dlist_peek_head(list); }
sys_dnode_t * sys_dlist_peek_head_not_empty__extern(const sys_dlist_t *list) { return sys_dlist_peek_head_not_empty(list); }
sys_dnode_t * sys_dlist_peek_next_no_check__extern(const sys_dlist_t *list, const sys_dnode_t *node) { return sys_dlist_peek_next_no_check(list, node); }
sys_dnode_t * sys_dlist_peek_next__extern(const sys_dlist_t *list, const sys_dnode_t *node) { return sys_dlist_peek_next(list, node); }
sys_dnode_t * sys_dlist_peek_prev_no_check__extern(const sys_dlist_t *list, const sys_dnode_t *node) { return sys_dlist_peek_prev_no_check(list, node); }
sys_dnode_t * sys_dlist_peek_prev__extern(const sys_dlist_t *list, const sys_dnode_t *node) { return sys_dlist_peek_prev(list, node); }
sys_dnode_t * sys_dlist_peek_tail__extern(const sys_dlist_t *list) { return sys_dlist_peek_tail(list); }
void sys_dlist_append__extern(sys_dlist_t *list, sys_dnode_t *node) { sys_dlist_append(list, node); }
void sys_dlist_prepend__extern(sys_dlist_t *list, sys_dnode_t *node) { sys_dlist_prepend(list, node); }
void sys_dlist_insert__extern(sys_dnode_t *successor, sys_dnode_t *node) { sys_dlist_insert(successor, node); }
void sys_dlist_insert_at__extern(sys_dlist_t *list, sys_dnode_t *node, int (*cond) (sys_dnode_t *node, void *data), void *data) { sys_dlist_insert_at(list, node, cond, data); }
void sys_dlist_dequeue__extern(sys_dnode_t *node) { sys_dlist_dequeue(node); }
void sys_dlist_remove__extern(sys_dnode_t *node) { sys_dlist_remove(node); }
void sys_dlist_range_prepend__extern(sys_dlist_t *dest, sys_dnode_t *start, sys_dnode_t *last) { sys_dlist_range_prepend(dest, start, last); }
void sys_dlist_range_append__extern(sys_dlist_t *dest, sys_dnode_t *start, sys_dnode_t *last) { sys_dlist_range_append(dest, start, last); }
sys_dnode_t * sys_dlist_get__extern(sys_dlist_t *list) { return sys_dlist_get(list); }
size_t sys_dlist_len__extern(const sys_dlist_t *list) { return sys_dlist_len(list); }
void sys_slist_init__extern(sys_slist_t *list) { sys_slist_init(list); }
sys_snode_t * sys_slist_peek_head__extern(const sys_slist_t *list) { return sys_slist_peek_head(list); }
sys_snode_t * sys_slist_peek_tail__extern(const sys_slist_t *list) { return sys_slist_peek_tail(list); }
bool sys_slist_is_empty__extern(const sys_slist_t *list) { return sys_slist_is_empty(list); }
sys_snode_t * sys_slist_peek_next_no_check__extern(const sys_snode_t *node) { return sys_slist_peek_next_no_check(node); }
sys_snode_t * sys_slist_peek_next__extern(const sys_snode_t *node) { return sys_slist_peek_next(node); }
void sys_slist_prepend__extern(sys_slist_t *list, sys_snode_t *node) { sys_slist_prepend(list, node); }
void sys_slist_append__extern(sys_slist_t *list, sys_snode_t *node) { sys_slist_append(list, node); }
void sys_slist_append_list__extern(sys_slist_t *list, void *head, void *tail) { sys_slist_append_list(list, head, tail); }
void sys_slist_merge_slist__extern(sys_slist_t *list, sys_slist_t *list_to_append) { sys_slist_merge_slist(list, list_to_append); }
void sys_slist_insert__extern(sys_slist_t *list, sys_snode_t *prev, sys_snode_t *node) { sys_slist_insert(list, prev, node); }
sys_snode_t * sys_slist_get_not_empty__extern(sys_slist_t *list) { return sys_slist_get_not_empty(list); }
sys_snode_t * sys_slist_get__extern(sys_slist_t *list) { return sys_slist_get(list); }
void sys_slist_remove__extern(sys_slist_t *list, sys_snode_t *prev_node, sys_snode_t *node) { sys_slist_remove(list, prev_node, node); }
bool sys_slist_find_and_remove__extern(sys_slist_t *list, sys_snode_t *node) { return sys_slist_find_and_remove(list, node); }
bool sys_slist_find__extern(const sys_slist_t *list, const sys_snode_t *node, sys_snode_t **prev) { return sys_slist_find(list, node, prev); }
size_t sys_slist_len__extern(const sys_slist_t *list) { return sys_slist_len(list); }
void sys_sflist_init__extern(sys_sflist_t *list) { sys_sflist_init(list); }
uint8_t sys_sfnode_flags_get__extern(const sys_sfnode_t *node) { return sys_sfnode_flags_get(node); }
sys_sfnode_t * sys_sflist_peek_head__extern(const sys_sflist_t *list) { return sys_sflist_peek_head(list); }
sys_sfnode_t * sys_sflist_peek_tail__extern(const sys_sflist_t *list) { return sys_sflist_peek_tail(list); }
void sys_sfnode_init__extern(sys_sfnode_t *node, uint8_t flags) { sys_sfnode_init(node, flags); }
void sys_sfnode_flags_set__extern(sys_sfnode_t *node, uint8_t flags) { sys_sfnode_flags_set(node, flags); }
bool sys_sflist_is_empty__extern(const sys_sflist_t *list) { return sys_sflist_is_empty(list); }
sys_sfnode_t * sys_sflist_peek_next_no_check__extern(const sys_sfnode_t *node) { return sys_sflist_peek_next_no_check(node); }
sys_sfnode_t * sys_sflist_peek_next__extern(const sys_sfnode_t *node) { return sys_sflist_peek_next(node); }
void sys_sflist_prepend__extern(sys_sflist_t *list, sys_sfnode_t *node) { sys_sflist_prepend(list, node); }
void sys_sflist_append__extern(sys_sflist_t *list, sys_sfnode_t *node) { sys_sflist_append(list, node); }
void sys_sflist_append_list__extern(sys_sflist_t *list, void *head, void *tail) { sys_sflist_append_list(list, head, tail); }
void sys_sflist_merge_sflist__extern(sys_sflist_t *list, sys_sflist_t *list_to_append) { sys_sflist_merge_sflist(list, list_to_append); }
void sys_sflist_insert__extern(sys_sflist_t *list, sys_sfnode_t *prev, sys_sfnode_t *node) { sys_sflist_insert(list, prev, node); }
sys_sfnode_t * sys_sflist_get_not_empty__extern(sys_sflist_t *list) { return sys_sflist_get_not_empty(list); }
sys_sfnode_t * sys_sflist_get__extern(sys_sflist_t *list) { return sys_sflist_get(list); }
void sys_sflist_remove__extern(sys_sflist_t *list, sys_sfnode_t *prev_node, sys_sfnode_t *node) { sys_sflist_remove(list, prev_node, node); }
bool sys_sflist_find_and_remove__extern(sys_sflist_t *list, sys_sfnode_t *node) { return sys_sflist_find_and_remove(list, node); }
size_t sys_sflist_len__extern(const sys_sflist_t *list) { return sys_sflist_len(list); }
bool sys_heap_validate__extern(struct sys_heap *heap) { return sys_heap_validate(heap); }
uintptr_t k_mem_phys_addr__extern(void *virt) { return k_mem_phys_addr(virt); }
void * k_mem_virt_addr__extern(uintptr_t phys) { return k_mem_virt_addr(phys); }
void * k_mem_map__extern(size_t size, uint32_t flags) { return k_mem_map(size, flags); }
void k_mem_unmap__extern(void *addr, size_t size) { k_mem_unmap(addr, size); }
void device_map__extern(mm_reg_t *virt_addr, uintptr_t phys_addr, size_t size, uint32_t flags) { device_map(virt_addr, phys_addr, size, flags); }
void device_unmap__extern(mm_reg_t virt_addr, size_t size) { device_unmap(virt_addr, size); }
device_handle_t device_handle_get__extern(const struct device *dev) { return device_handle_get(dev); }
const struct device * device_from_handle__extern(device_handle_t dev_handle) { return device_from_handle(dev_handle); }
const struct device * device_get_binding__extern(const char *name) { return device_get_binding(name); }
bool device_is_ready__extern(const struct device *dev) { return device_is_ready(dev); }
int device_init__extern(const struct device *dev) { return device_init(dev); }
int device_deinit__extern(const struct device *dev) { return device_deinit(dev); }
const struct device * device_get_by_dt_nodelabel__extern(const char *nodelabel) { return device_get_by_dt_nodelabel(nodelabel); }
uint8_t sys_read8__extern(mem_addr_t addr) { return sys_read8(addr); }
void sys_write8__extern(uint8_t data, mem_addr_t addr) { sys_write8(data, addr); }
uint16_t sys_read16__extern(mem_addr_t addr) { return sys_read16(addr); }
void sys_write16__extern(uint16_t data, mem_addr_t addr) { sys_write16(data, addr); }
uint32_t sys_read32__extern(mem_addr_t addr) { return sys_read32(addr); }
void sys_write32__extern(uint32_t data, mem_addr_t addr) { sys_write32(data, addr); }
uint64_t sys_read64__extern(mem_addr_t addr) { return sys_read64(addr); }
void sys_write64__extern(uint64_t data, mem_addr_t addr) { sys_write64(data, addr); }
void sys_set_bit__extern(mem_addr_t addr, unsigned int bit) { sys_set_bit(addr, bit); }
void sys_clear_bit__extern(mem_addr_t addr, unsigned int bit) { sys_clear_bit(addr, bit); }
int sys_test_bit__extern(mem_addr_t addr, unsigned int bit) { return sys_test_bit(addr, bit); }
void sys_set_bits__extern(mem_addr_t addr, unsigned int mask) { sys_set_bits(addr, mask); }
void sys_clear_bits__extern(mem_addr_t addr, unsigned int mask) { sys_clear_bits(addr, mask); }
void sys_bitfield_set_bit__extern(mem_addr_t addr, unsigned int bit) { sys_bitfield_set_bit(addr, bit); }
void sys_bitfield_clear_bit__extern(mem_addr_t addr, unsigned int bit) { sys_bitfield_clear_bit(addr, bit); }
int sys_bitfield_test_bit__extern(mem_addr_t addr, unsigned int bit) { return sys_bitfield_test_bit(addr, bit); }
int sys_test_and_set_bit__extern(mem_addr_t addr, unsigned int bit) { return sys_test_and_set_bit(addr, bit); }
int sys_test_and_clear_bit__extern(mem_addr_t addr, unsigned int bit) { return sys_test_and_clear_bit(addr, bit); }
int sys_bitfield_test_and_set_bit__extern(mem_addr_t addr, unsigned int bit) { return sys_bitfield_test_and_set_bit(addr, bit); }
int sys_bitfield_test_and_clear_bit__extern(mem_addr_t addr, unsigned int bit) { return sys_bitfield_test_and_clear_bit(addr, bit); }
int sys_timepoint_cmp__extern(k_timepoint_t a, k_timepoint_t b) { return sys_timepoint_cmp(a, b); }
bool sys_timepoint_expired__extern(k_timepoint_t timepoint) { return sys_timepoint_expired(timepoint); }
void sys_clock_getrtoffset__extern(struct timespec *tp) { sys_clock_getrtoffset(tp); }
int sys_clock_settime__extern(int clock_id, const struct timespec *tp) { return sys_clock_settime(clock_id, tp); }
int sys_clock_nanosleep__extern(int clock_id, int flags, const struct timespec *rqtp, struct timespec *rmtp) { return sys_clock_nanosleep(clock_id, flags, rqtp, rmtp); }
k_spinlock_key_t k_spin_lock__extern(struct k_spinlock *l) { return k_spin_lock(l); }
int k_spin_trylock__extern(struct k_spinlock *l, k_spinlock_key_t *k) { return k_spin_trylock(l, k); }
void k_spin_unlock__extern(struct k_spinlock *l, k_spinlock_key_t key) { k_spin_unlock(l, key); }
void k_spin_release__extern(struct k_spinlock *l) { k_spin_release(l); }
char * K_KERNEL_STACK_BUFFER__extern(k_thread_stack_t *sym) { return K_KERNEL_STACK_BUFFER(sym); }
void k_object_init__extern(const void *obj) { k_object_init(obj); }
struct k_object * k_object_create_dynamic_aligned__extern(size_t align, size_t size) { return k_object_create_dynamic_aligned(align, size); }
struct k_object * k_object_create_dynamic__extern(size_t size) { return k_object_create_dynamic(size); }
void k_object_access_revoke__extern(const void *object, struct k_thread *thread) { k_object_access_revoke(object, thread); }
void k_object_access_all_grant__extern(const void *object) { k_object_access_all_grant(object); }
bool k_object_is_valid__extern(const void *obj, enum k_objects otype) { return k_object_is_valid(obj, otype); }
void k_object_free__extern(void *obj) { k_object_free(obj); }
void k_object_access_grant__extern(const void *object, struct k_thread *thread) { k_object_access_grant(object, thread); }
void k_object_release__extern(const void *object) { k_object_release(object); }
int k_object_access_check__extern(const void *object) { return k_object_access_check(object); }
void * k_object_alloc__extern(enum k_objects otype) { return k_object_alloc(otype); }
void * k_object_alloc_size__extern(enum k_objects otype, size_t size) { return k_object_alloc_size(otype, size); }
void k_thread_runtime_stats_longest_frame_reset__extern(struct k_thread *thread) { k_thread_runtime_stats_longest_frame_reset(thread); }
void k_thread_foreach_filter_by_cpu__extern(unsigned int cpu, k_thread_user_cb_t user_cb, void *user_data) { k_thread_foreach_filter_by_cpu(cpu, user_cb, user_data); }
void k_thread_foreach_unlocked_filter_by_cpu__extern(unsigned int cpu, k_thread_user_cb_t user_cb, void *user_data) { k_thread_foreach_unlocked_filter_by_cpu(cpu, user_cb, user_data); }
k_thread_stack_t * k_thread_stack_alloc__extern(size_t size, int flags) { return k_thread_stack_alloc(size, flags); }
int k_thread_stack_free__extern(k_thread_stack_t *stack) { return k_thread_stack_free(stack); }
k_tid_t k_thread_create__extern(struct k_thread *new_thread, k_thread_stack_t *stack, size_t stack_size, k_thread_entry_t entry, void *p1, void *p2, void *p3, int prio, uint32_t options, k_timeout_t delay) { return k_thread_create(new_thread, stack, stack_size, entry, p1, p2, p3, prio, options, delay); }
void k_thread_heap_assign__extern(struct k_thread *thread, struct k_heap *heap) { k_thread_heap_assign(thread, heap); }
int k_thread_join__extern(struct k_thread *thread, k_timeout_t timeout) { return k_thread_join(thread, timeout); }
int32_t k_sleep__extern(k_timeout_t timeout) { return k_sleep(timeout); }
int32_t k_msleep__extern(int32_t ms) { return k_msleep(ms); }
int32_t k_usleep__extern(int32_t us) { return k_usleep(us); }
void k_busy_wait__extern(uint32_t usec_to_wait) { k_busy_wait(usec_to_wait); }
void k_yield__extern(void) { k_yield(); }
void k_wakeup__extern(k_tid_t thread) { k_wakeup(thread); }
k_tid_t k_sched_current_thread_query__extern(void) { return k_sched_current_thread_query(); }
bool k_is_pre_kernel__extern(void) { return k_is_pre_kernel(); }
k_tid_t k_current_get__extern(void) { return k_current_get(); }
void k_thread_abort__extern(k_tid_t thread) { k_thread_abort(thread); }
k_ticks_t k_thread_timeout_expires_ticks__extern(const struct k_thread *thread) { return k_thread_timeout_expires_ticks(thread); }
k_ticks_t k_thread_timeout_remaining_ticks__extern(const struct k_thread *thread) { return k_thread_timeout_remaining_ticks(thread); }
int k_thread_priority_get__extern(k_tid_t thread) { return k_thread_priority_get(thread); }
void k_thread_priority_set__extern(k_tid_t thread, int prio) { k_thread_priority_set(thread, prio); }
void k_reschedule__extern(void) { k_reschedule(); }
void k_thread_suspend__extern(k_tid_t thread) { k_thread_suspend(thread); }
void k_thread_resume__extern(k_tid_t thread) { k_thread_resume(thread); }
void k_thread_start__extern(k_tid_t thread) { k_thread_start(thread); }
int k_is_preempt_thread__extern(void) { return k_is_preempt_thread(); }
void k_thread_custom_data_set__extern(void *value) { k_thread_custom_data_set(value); }
void * k_thread_custom_data_get__extern(void) { return k_thread_custom_data_get(); }
int k_thread_name_set__extern(k_tid_t thread, const char *str) { return k_thread_name_set(thread, str); }
int k_thread_name_copy__extern(k_tid_t thread, char *buf, size_t size) { return k_thread_name_copy(thread, buf, size); }
void k_timer_start__extern(struct k_timer *timer, k_timeout_t duration, k_timeout_t period) { k_timer_start(timer, duration, period); }
void k_timer_stop__extern(struct k_timer *timer) { k_timer_stop(timer); }
uint32_t k_timer_status_get__extern(struct k_timer *timer) { return k_timer_status_get(timer); }
uint32_t k_timer_status_sync__extern(struct k_timer *timer) { return k_timer_status_sync(timer); }
k_ticks_t k_timer_expires_ticks__extern(const struct k_timer *timer) { return k_timer_expires_ticks(timer); }
k_ticks_t k_timer_remaining_ticks__extern(const struct k_timer *timer) { return k_timer_remaining_ticks(timer); }
uint32_t k_timer_remaining_get__extern(struct k_timer *timer) { return k_timer_remaining_get(timer); }
void k_timer_user_data_set__extern(struct k_timer *timer, void *user_data) { k_timer_user_data_set(timer, user_data); }
void * k_timer_user_data_get__extern(const struct k_timer *timer) { return k_timer_user_data_get(timer); }
int64_t k_uptime_ticks__extern(void) { return k_uptime_ticks(); }
int64_t k_uptime_get__extern(void) { return k_uptime_get(); }
uint32_t k_uptime_get_32__extern(void) { return k_uptime_get_32(); }
uint32_t k_uptime_seconds__extern(void) { return k_uptime_seconds(); }
int64_t k_uptime_delta__extern(int64_t *reftime) { return k_uptime_delta(reftime); }
uint32_t k_cycle_get_32__extern(void) { return k_cycle_get_32(); }
uint64_t k_cycle_get_64__extern(void) { return k_cycle_get_64(); }
void k_queue_init__extern(struct k_queue *queue) { k_queue_init(queue); }
void k_queue_cancel_wait__extern(struct k_queue *queue) { k_queue_cancel_wait(queue); }
int32_t k_queue_alloc_append__extern(struct k_queue *queue, void *data) { return k_queue_alloc_append(queue, data); }
int32_t k_queue_alloc_prepend__extern(struct k_queue *queue, void *data) { return k_queue_alloc_prepend(queue, data); }
void * k_queue_get__extern(struct k_queue *queue, k_timeout_t timeout) { return k_queue_get(queue, timeout); }
int k_queue_is_empty__extern(struct k_queue *queue) { return k_queue_is_empty(queue); }
void * k_queue_peek_head__extern(struct k_queue *queue) { return k_queue_peek_head(queue); }
void * k_queue_peek_tail__extern(struct k_queue *queue) { return k_queue_peek_tail(queue); }
void k_event_init__extern(struct k_event *event) { k_event_init(event); }
uint32_t k_event_post__extern(struct k_event *event, uint32_t events) { return k_event_post(event, events); }
uint32_t k_event_set__extern(struct k_event *event, uint32_t events) { return k_event_set(event, events); }
uint32_t k_event_set_masked__extern(struct k_event *event, uint32_t events, uint32_t events_mask) { return k_event_set_masked(event, events, events_mask); }
uint32_t k_event_clear__extern(struct k_event *event, uint32_t events) { return k_event_clear(event, events); }
uint32_t k_event_wait__extern(struct k_event *event, uint32_t events, bool reset, k_timeout_t timeout) { return k_event_wait(event, events, reset, timeout); }
uint32_t k_event_wait_all__extern(struct k_event *event, uint32_t events, bool reset, k_timeout_t timeout) { return k_event_wait_all(event, events, reset, timeout); }
uint32_t k_event_wait_safe__extern(struct k_event *event, uint32_t events, bool reset, k_timeout_t timeout) { return k_event_wait_safe(event, events, reset, timeout); }
uint32_t k_event_wait_all_safe__extern(struct k_event *event, uint32_t events, bool reset, k_timeout_t timeout) { return k_event_wait_all_safe(event, events, reset, timeout); }
uint32_t k_event_test__extern(struct k_event *event, uint32_t events_mask) { return k_event_test(event, events_mask); }
int32_t k_stack_alloc_init__extern(struct k_stack *stack, uint32_t num_entries) { return k_stack_alloc_init(stack, num_entries); }
int k_stack_push__extern(struct k_stack *stack, stack_data_t data) { return k_stack_push(stack, data); }
int k_stack_pop__extern(struct k_stack *stack, stack_data_t *data, k_timeout_t timeout) { return k_stack_pop(stack, data, timeout); }
int k_mutex_init__extern(struct k_mutex *mutex) { return k_mutex_init(mutex); }
int k_mutex_lock__extern(struct k_mutex *mutex, k_timeout_t timeout) { return k_mutex_lock(mutex, timeout); }
int k_mutex_unlock__extern(struct k_mutex *mutex) { return k_mutex_unlock(mutex); }
int k_condvar_init__extern(struct k_condvar *condvar) { return k_condvar_init(condvar); }
int k_condvar_signal__extern(struct k_condvar *condvar) { return k_condvar_signal(condvar); }
int k_condvar_broadcast__extern(struct k_condvar *condvar) { return k_condvar_broadcast(condvar); }
int k_condvar_wait__extern(struct k_condvar *condvar, struct k_mutex *mutex, k_timeout_t timeout) { return k_condvar_wait(condvar, mutex, timeout); }
int k_sem_init__extern(struct k_sem *sem, unsigned int initial_count, unsigned int limit) { return k_sem_init(sem, initial_count, limit); }
int k_sem_take__extern(struct k_sem *sem, k_timeout_t timeout) { return k_sem_take(sem, timeout); }
void k_sem_give__extern(struct k_sem *sem) { k_sem_give(sem); }
void k_sem_reset__extern(struct k_sem *sem) { k_sem_reset(sem); }
unsigned int k_sem_count_get__extern(struct k_sem *sem) { return k_sem_count_get(sem); }
bool k_work_is_pending__extern(const struct k_work *work) { return k_work_is_pending(work); }
k_tid_t k_work_queue_thread_get__extern(struct k_work_q *queue) { return k_work_queue_thread_get(queue); }
struct k_work_delayable * k_work_delayable_from_work__extern(struct k_work *work) { return k_work_delayable_from_work(work); }
bool k_work_delayable_is_pending__extern(const struct k_work_delayable *dwork) { return k_work_delayable_is_pending(dwork); }
k_ticks_t k_work_delayable_expires_get__extern(const struct k_work_delayable *dwork) { return k_work_delayable_expires_get(dwork); }
k_ticks_t k_work_delayable_remaining_get__extern(const struct k_work_delayable *dwork) { return k_work_delayable_remaining_get(dwork); }
void k_work_user_init__extern(struct k_work_user *work, k_work_user_handler_t handler) { k_work_user_init(work, handler); }
bool k_work_user_is_pending__extern(struct k_work_user *work) { return k_work_user_is_pending(work); }
int k_work_user_submit_to_queue__extern(struct k_work_user_q *work_q, struct k_work_user *work) { return k_work_user_submit_to_queue(work_q, work); }
k_tid_t k_work_user_queue_thread_get__extern(struct k_work_user_q *work_q) { return k_work_user_queue_thread_get(work_q); }
int k_msgq_alloc_init__extern(struct k_msgq *msgq, size_t msg_size, uint32_t max_msgs) { return k_msgq_alloc_init(msgq, msg_size, max_msgs); }
int k_msgq_put__extern(struct k_msgq *msgq, const void *data, k_timeout_t timeout) { return k_msgq_put(msgq, data, timeout); }
int k_msgq_put_front__extern(struct k_msgq *msgq, const void *data) { return k_msgq_put_front(msgq, data); }
int k_msgq_get__extern(struct k_msgq *msgq, void *data, k_timeout_t timeout) { return k_msgq_get(msgq, data, timeout); }
int k_msgq_peek__extern(struct k_msgq *msgq, void *data) { return k_msgq_peek(msgq, data); }
int k_msgq_peek_at__extern(struct k_msgq *msgq, void *data, uint32_t idx) { return k_msgq_peek_at(msgq, data, idx); }
void k_msgq_purge__extern(struct k_msgq *msgq) { k_msgq_purge(msgq); }
uint32_t k_msgq_num_free_get__extern(struct k_msgq *msgq) { return k_msgq_num_free_get(msgq); }
void k_msgq_get_attrs__extern(struct k_msgq *msgq, struct k_msgq_attrs *attrs) { k_msgq_get_attrs(msgq, attrs); }
uint32_t k_msgq_num_used_get__extern(struct k_msgq *msgq) { return k_msgq_num_used_get(msgq); }
void k_pipe_init__extern(struct k_pipe *pipe, uint8_t *buffer, size_t buffer_size) { k_pipe_init(pipe, buffer, buffer_size); }
int k_pipe_write__extern(struct k_pipe *pipe, const uint8_t *data, size_t len, k_timeout_t timeout) { return k_pipe_write(pipe, data, len, timeout); }
int k_pipe_read__extern(struct k_pipe *pipe, uint8_t *data, size_t len, k_timeout_t timeout) { return k_pipe_read(pipe, data, len, timeout); }
void k_pipe_reset__extern(struct k_pipe *pipe) { k_pipe_reset(pipe); }
void k_pipe_close__extern(struct k_pipe *pipe) { k_pipe_close(pipe); }
uint32_t k_mem_slab_num_used_get__extern(struct k_mem_slab *slab) { return k_mem_slab_num_used_get(slab); }
uint32_t k_mem_slab_max_used_get__extern(struct k_mem_slab *slab) { return k_mem_slab_max_used_get(slab); }
uint32_t k_mem_slab_num_free_get__extern(struct k_mem_slab *slab) { return k_mem_slab_num_free_get(slab); }
int k_poll__extern(struct k_poll_event *events, int num_events, k_timeout_t timeout) { return k_poll(events, num_events, timeout); }
void k_poll_signal_init__extern(struct k_poll_signal *sig) { k_poll_signal_init(sig); }
void k_poll_signal_reset__extern(struct k_poll_signal *sig) { k_poll_signal_reset(sig); }
void k_poll_signal_check__extern(struct k_poll_signal *sig, unsigned int *signaled, int *result) { k_poll_signal_check(sig, signaled, result); }
int k_poll_signal_raise__extern(struct k_poll_signal *sig, int result) { return k_poll_signal_raise(sig, result); }
void k_cpu_idle__extern(void) { k_cpu_idle(); }
void k_cpu_atomic_idle__extern(unsigned int key) { k_cpu_atomic_idle(key); }
void k_str_out__extern(char *c, size_t n) { k_str_out(c, n); }
int k_float_disable__extern(struct k_thread *thread) { return k_float_disable(thread); }
int k_float_enable__extern(struct k_thread *thread, unsigned int options) { return k_float_enable(thread, options); }
int k_thread_stack_space_get__extern(const struct k_thread *thread, size_t *unused_ptr) { return k_thread_stack_space_get(thread, unused_ptr); }
int k_thread_runtime_stack_unused_threshold_pct_set__extern(struct k_thread *thread, uint32_t pct) { return k_thread_runtime_stack_unused_threshold_pct_set(thread, pct); }
int k_thread_runtime_stack_unused_threshold_set__extern(struct k_thread *thread, size_t threshold) { return k_thread_runtime_stack_unused_threshold_set(thread, threshold); }
size_t k_thread_runtime_stack_unused_threshold_get__extern(struct k_thread *thread) { return k_thread_runtime_stack_unused_threshold_get(thread); }
void k_thread_deadline_set__extern(k_tid_t thread, int deadline) { k_thread_deadline_set(thread, deadline); }
void k_thread_absolute_deadline_set__extern(k_tid_t thread, int deadline) { k_thread_absolute_deadline_set(thread, deadline); }
int k_futex_wait__extern(struct k_futex *futex, int expected, k_timeout_t timeout) { return k_futex_wait(futex, expected, timeout); }
int k_futex_wake__extern(struct k_futex *futex, bool wake_all) { return k_futex_wake(futex, wake_all); }
bool gpio_is_ready_dt__extern(const struct gpio_dt_spec *spec) { return gpio_is_ready_dt(spec); }
int gpio_pin_interrupt_configure__extern(const struct device *port, gpio_pin_t pin, gpio_flags_t flags) { return gpio_pin_interrupt_configure(port, pin, flags); }
int gpio_pin_interrupt_configure_dt__extern(const struct gpio_dt_spec *spec, gpio_flags_t flags) { return gpio_pin_interrupt_configure_dt(spec, flags); }
int gpio_pin_configure__extern(const struct device *port, gpio_pin_t pin, gpio_flags_t flags) { return gpio_pin_configure(port, pin, flags); }
int gpio_pin_configure_dt__extern(const struct gpio_dt_spec *spec, gpio_flags_t extra_flags) { return gpio_pin_configure_dt(spec, extra_flags); }
int gpio_port_get_direction__extern(const struct device *port, gpio_port_pins_t map, gpio_port_pins_t *inputs, gpio_port_pins_t *outputs) { return gpio_port_get_direction(port, map, inputs, outputs); }
int gpio_pin_is_input__extern(const struct device *port, gpio_pin_t pin) { return gpio_pin_is_input(port, pin); }
int gpio_pin_is_input_dt__extern(const struct gpio_dt_spec *spec) { return gpio_pin_is_input_dt(spec); }
int gpio_pin_is_output__extern(const struct device *port, gpio_pin_t pin) { return gpio_pin_is_output(port, pin); }
int gpio_pin_is_output_dt__extern(const struct gpio_dt_spec *spec) { return gpio_pin_is_output_dt(spec); }
int gpio_pin_get_config__extern(const struct device *port, gpio_pin_t pin, gpio_flags_t *flags) { return gpio_pin_get_config(port, pin, flags); }
int gpio_pin_get_config_dt__extern(const struct gpio_dt_spec *spec, gpio_flags_t *flags) { return gpio_pin_get_config_dt(spec, flags); }
int gpio_port_get_raw__extern(const struct device *port, gpio_port_value_t *value) { return gpio_port_get_raw(port, value); }
int gpio_port_get__extern(const struct device *port, gpio_port_value_t *value) { return gpio_port_get(port, value); }
int gpio_port_set_masked_raw__extern(const struct device *port, gpio_port_pins_t mask, gpio_port_value_t value) { return gpio_port_set_masked_raw(port, mask, value); }
int gpio_port_set_masked__extern(const struct device *port, gpio_port_pins_t mask, gpio_port_value_t value) { return gpio_port_set_masked(port, mask, value); }
int gpio_port_set_bits_raw__extern(const struct device *port, gpio_port_pins_t pins) { return gpio_port_set_bits_raw(port, pins); }
int gpio_port_set_bits__extern(const struct device *port, gpio_port_pins_t pins) { return gpio_port_set_bits(port, pins); }
int gpio_port_clear_bits_raw__extern(const struct device *port, gpio_port_pins_t pins) { return gpio_port_clear_bits_raw(port, pins); }
int gpio_port_clear_bits__extern(const struct device *port, gpio_port_pins_t pins) { return gpio_port_clear_bits(port, pins); }
int gpio_port_toggle_bits__extern(const struct device *port, gpio_port_pins_t pins) { return gpio_port_toggle_bits(port, pins); }
int gpio_port_set_clr_bits_raw__extern(const struct device *port, gpio_port_pins_t set_pins, gpio_port_pins_t clear_pins) { return gpio_port_set_clr_bits_raw(port, set_pins, clear_pins); }
int gpio_port_set_clr_bits__extern(const struct device *port, gpio_port_pins_t set_pins, gpio_port_pins_t clear_pins) { return gpio_port_set_clr_bits(port, set_pins, clear_pins); }
int gpio_pin_get_raw__extern(const struct device *port, gpio_pin_t pin) { return gpio_pin_get_raw(port, pin); }
int gpio_pin_get__extern(const struct device *port, gpio_pin_t pin) { return gpio_pin_get(port, pin); }
int gpio_pin_get_dt__extern(const struct gpio_dt_spec *spec) { return gpio_pin_get_dt(spec); }
int gpio_pin_set_raw__extern(const struct device *port, gpio_pin_t pin, int value) { return gpio_pin_set_raw(port, pin, value); }
int gpio_pin_set__extern(const struct device *port, gpio_pin_t pin, int value) { return gpio_pin_set(port, pin, value); }
int gpio_pin_set_dt__extern(const struct gpio_dt_spec *spec, int value) { return gpio_pin_set_dt(spec, value); }
int gpio_pin_toggle__extern(const struct device *port, gpio_pin_t pin) { return gpio_pin_toggle(port, pin); }
int gpio_pin_toggle_dt__extern(const struct gpio_dt_spec *spec) { return gpio_pin_toggle_dt(spec); }
void gpio_init_callback__extern(struct gpio_callback *callback, gpio_callback_handler_t handler, gpio_port_pins_t pin_mask) { gpio_init_callback(callback, handler, pin_mask); }
int gpio_add_callback__extern(const struct device *port, struct gpio_callback *callback) { return gpio_add_callback(port, callback); }
int gpio_add_callback_dt__extern(const struct gpio_dt_spec *spec, struct gpio_callback *callback) { return gpio_add_callback_dt(spec, callback); }
int gpio_remove_callback__extern(const struct device *port, struct gpio_callback *callback) { return gpio_remove_callback(port, callback); }
int gpio_remove_callback_dt__extern(const struct gpio_dt_spec *spec, struct gpio_callback *callback) { return gpio_remove_callback_dt(spec, callback); }
int gpio_get_pending_int__extern(const struct device *dev) { return gpio_get_pending_int(dev); }
void z_log_msg_simple_create_0__extern(const void *source, uint32_t level, const char *fmt) { z_log_msg_simple_create_0(source, level, fmt); }
void z_log_msg_simple_create_1__extern(const void *source, uint32_t level, const char *fmt, uint32_t arg) { z_log_msg_simple_create_1(source, level, fmt, arg); }
void z_log_msg_simple_create_2__extern(const void *source, uint32_t level, const char *fmt, uint32_t arg0, uint32_t arg1) { z_log_msg_simple_create_2(source, level, fmt, arg0, arg1); }
void z_log_msg_static_create__extern(const void *source, const struct log_msg_desc desc, uint8_t *package, const void *data) { z_log_msg_static_create(source, desc, package, data); }
bool z_log_item_is_msg__extern(const union log_msg_generic *msg) { return z_log_item_is_msg(msg); }
char z_log_minimal_level_to_char__extern(int level) { return z_log_minimal_level_to_char(level); }
int bt_addr_cmp__extern(const bt_addr_t *a, const bt_addr_t *b) { return bt_addr_cmp(a, b); }
bool bt_addr_eq__extern(const bt_addr_t *a, const bt_addr_t *b) { return bt_addr_eq(a, b); }
int bt_addr_le_cmp__extern(const bt_addr_le_t *a, const bt_addr_le_t *b) { return bt_addr_le_cmp(a, b); }
bool bt_addr_le_eq__extern(const bt_addr_le_t *a, const bt_addr_le_t *b) { return bt_addr_le_eq(a, b); }
void bt_addr_copy__extern(bt_addr_t *dst, const bt_addr_t *src) { bt_addr_copy(dst, src); }
void bt_addr_le_copy__extern(bt_addr_le_t *dst, const bt_addr_le_t *src) { bt_addr_le_copy(dst, src); }
bool bt_addr_le_is_rpa__extern(const bt_addr_le_t *addr) { return bt_addr_le_is_rpa(addr); }
bool bt_addr_le_is_identity__extern(const bt_addr_le_t *addr) { return bt_addr_le_is_identity(addr); }
int bt_addr_to_str__extern(const bt_addr_t *addr, char *str, size_t len) { return bt_addr_to_str(addr, str, len); }
int bt_addr_le_to_str__extern(const bt_addr_le_t *addr, char *str, size_t len) { return bt_addr_le_to_str(addr, str, len); }
int flash_params_get_erase_cap__extern(const struct flash_parameters *p) { return flash_params_get_erase_cap(p); }
int flash_read__extern(const struct device *dev, off_t offset, void *data, size_t len) { return flash_read(dev, offset, data, len); }
int flash_write__extern(const struct device *dev, off_t offset, const void *data, size_t len) { return flash_write(dev, offset, data, len); }
int flash_erase__extern(const struct device *dev, off_t offset, size_t size) { return flash_erase(dev, offset, size); }
int flash_get_size__extern(const struct device *dev, uint64_t *size) { return flash_get_size(dev, size); }
int flash_fill__extern(const struct device *dev, uint8_t val, off_t offset, size_t size) { return flash_fill(dev, val, offset, size); }
int flash_flatten__extern(const struct device *dev, off_t offset, size_t size) { return flash_flatten(dev, offset, size); }
size_t flash_get_write_block_size__extern(const struct device *dev) { return flash_get_write_block_size(dev); }
const struct flash_parameters * flash_get_parameters__extern(const struct device *dev) { return flash_get_parameters(dev); }
int flash_ex_op__extern(const struct device *dev, uint16_t code, const uintptr_t in, void *out) { return flash_ex_op(dev, code, in, out); }
int flash_copy__extern(const struct device *src_dev, off_t src_offset, const struct device *dst_dev, off_t dst_offset, off_t size, uint8_t *buf, size_t buf_size) { return flash_copy(src_dev, src_offset, dst_dev, dst_offset, size, buf, buf_size); }
int flash_get_page_info_by_offs__extern(const struct device *dev, off_t offset, struct flash_pages_info *info) { return flash_get_page_info_by_offs(dev, offset, info); }
int flash_get_page_info_by_idx__extern(const struct device *dev, uint32_t page_index, struct flash_pages_info *info) { return flash_get_page_info_by_idx(dev, page_index, info); }
size_t flash_get_page_count__extern(const struct device *dev) { return flash_get_page_count(dev); }
int flash_sfdp_read__extern(const struct device *dev, off_t offset, void *data, size_t len) { return flash_sfdp_read(dev, offset, data, len); }
int flash_read_jedec_id__extern(const struct device *dev, uint8_t *id) { return flash_read_jedec_id(dev, id); }
int zr_irq_lock__extern(void) { return zr_irq_lock(); }
void zr_irq_unlock__extern(int key) { zr_irq_unlock(key); }
