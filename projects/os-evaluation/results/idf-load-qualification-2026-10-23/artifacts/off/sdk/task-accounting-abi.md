# Generated task-accounting declarations

Exact excerpts from the generated results/idf-load-qualification-2026-10-23/artifacts/off/sdk/bindings.rs input (SHA-256
8e737b7c847c0e12fbbc4f163be3ae9dbecacc29bb3f9a50480c60c3e3df1780). Byte ranges are zero-based, end-exclusive UTF-8 offsets.
The full generated file is retired; these excerpts retain the task counter
and snapshot pointer types used to inspect this configuration. They are not a
standalone binding module, a complete SDK ABI audit or a build reproduction.

## pub type UBaseType_t

Bytes 953841..953890:

    pub type UBaseType_t = :: core :: ffi :: c_uint ;

## pub type TaskHandle_t

Bytes 971055..971106:

    pub type TaskHandle_t = * mut tskTaskControlBlock ;

## pub type eTaskState

Bytes 972154..972202:

    pub type eTaskState = :: core :: ffi :: c_uint ;

## pub struct xTASK_STATUS

Bytes 974498..976106:

    # [repr (C)] # [derive (Debug , Copy , Clone)] pub struct xTASK_STATUS { # [doc = "< The handle of the task to which the rest of the information in the structure relates."] pub xHandle : TaskHandle_t , # [doc = "< A pointer to the task's name.  This value will be invalid if the task was deleted since the structure was populated!"] pub pcTaskName : * const :: core :: ffi :: c_char , # [doc = "< A number unique to the task."] pub xTaskNumber : UBaseType_t , # [doc = "< The state in which the task existed when the structure was populated."] pub eCurrentState : eTaskState , # [doc = "< The priority at which the task was running (may be inherited) when the structure was populated."] pub uxCurrentPriority : UBaseType_t , # [doc = "< The priority to which the task will return if the task's current priority has been inherited to avoid unbounded priority inversion when obtaining a mutex.  Only valid if configUSE_MUTEXES is defined as 1 in FreeRTOSConfig.h."] pub uxBasePriority : UBaseType_t , # [doc = "< The total run time allocated to the task so far, as defined by the run time stats clock.  See https://www.FreeRTOS.org/rtos-run-time-stats.html.  Only valid when configGENERATE_RUN_TIME_STATS is defined as 1 in FreeRTOSConfig.h."] pub ulRunTimeCounter : u32 , # [doc = "< Points to the lowest address of the task's stack area."] pub pxStackBase : * mut StackType_t , # [doc = "< The minimum amount of stack space that has remained for the task since the task was created.  The closer this value is to zero the closer the task has come to overflowing its stack."] pub usStackHighWaterMark : u32 , }

## pub type TaskStatus_t

Bytes 976435..976473:

    pub type TaskStatus_t = xTASK_STATUS ;

## pub fn uxTaskGetSystemState

Bytes 1012042..1012185:

    pub fn uxTaskGetSystemState (pxTaskStatusArray : * mut TaskStatus_t , uxArraySize : UBaseType_t , pulTotalRunTime : * mut u32) -> UBaseType_t ;
