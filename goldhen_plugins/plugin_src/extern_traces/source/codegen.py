from mako.template import Template

imports = [
    # ("sceAppContentAppParamGetInt", False),
    # ("sceAppContentInitialize", False),
    # ("sceAppContentTemporaryDataGetAvailableSpaceKb", False),
    # ("sceAppContentGetEntitlementKey", False),
    # ("sceAppContentTemporaryDataFormat", False),
    # ("sceAppContentTemporaryDataMount2", False),
    # ("sceAppContentGetAddcontInfoList", False),
    # ("sceAudioInOpen", False),
    # # ("sceAudioInInput", False),
    ("sceAudioOutInit", 0x1c56d88),
    ("sceAudioOutOutput", 0x1c56e08),
    ("sceAudioOutOpen", 0x1c56e10),
    ("sceAudioOutClose", 0x1c56de8),
    # ("sceAvPlayerPause", False),
    # ("sceAvPlayerStart", False),
    # ("sceAvPlayerGetVideoDataEx", False),
    # ("sceAvPlayerAddSource", False),
    # ("sceAvPlayerClose", False),
    # ("sceAvPlayerEnableStream", False),
    # ("sceAvPlayerSetLooping", False),
    # ("sceAvPlayerIsActive", False),
    # ("sceAvPlayerGetAudioData", False),
    # ("sceAvPlayerJumpToTime", False),
    # ("sceAvPlayerInit", False),
    # ("sceAvPlayerGetStreamInfo", False),
    # ("sceAvPlayerStreamCount", False),
    # ("sceAvPlayerResume", False),
    # ("sceAvPlayerCurrentTime", False),
    # ("sceCameraStop", False),
    # ("sceCameraStart", False),
    # ("sceCameraOpen", False),
    # ("sceCameraClose", False),
    # ("sceCameraSetConfig", False),
    # ("sceCameraGetConfig", False),
    # ("sceCameraSetAttribute", False),
    # ("sceCameraGetFrameData", False),
    # # ("sceCameraIsAttached", False),
    # ("sceCommonDialogInitialize", False),
    # ("sceCompanionHttpdStop", False),
    # ("sceCompanionHttpdAddHeader", False),
    # ("sceCompanionHttpdRegisterRequestCallback", False),
    # ("sceCompanionHttpdStart", False),
    # ("sceCompanionHttpdOptParamInitialize", False),
    # ("sceCompanionHttpdSetStatus", False),
    # ("sceCompanionHttpdInitialize", False),
    # ("sceContentSearchTerm", False),
    # ("sceContentSearchGetMyApplicationIndex", False),
    # ("sceContentSearchGetContentLastUpdateId", False),
    # ("sceContentSearchSearchContent", False),
    # ("sceContentSearchInit", False),
    # ("sceErrorDialogTerminate", False),
    # ("sceErrorDialogInitialize", False),
    # ("sceErrorDialogOpen", False),
    # ("sceErrorDialogUpdateStatus", False),
    # # ("sceFiberReturnToThread", False),
    # # ("sceFiberFinalize", False),
    # # ("sceFiberRun", False),
    # # ("sceFiberOptParamInitialize", False),
    # # ("_sceFiberInitializeImpl", False),
    # # ("sceFiosOpGetActualCount", False),
    # # ("sceFiosDHCloseSync", False),
    # # ("sceFiosOpDelete", False),
    # # ("sceFiosFHCloseSync", False),
    # # ("sceFiosFHReadSync", False),
    # # ("sceFiosFHSyncSync", False),
    # # ("sceFiosFHGetSize", False),
    # # ("sceFiosRenameSync", False),
    # # ("sceFiosDHOpenSync", False),
    # # ("sceFiosFHWriteSync", False),
    # # ("sceFiosDeleteSync", False),
    # # ("sceFiosFileExistsSync", False),
    # # ("sceFiosDirectoryExistsSync", False),
    # # ("sceFiosOpWait", False),
    # # ("sceFiosDirectoryCreateWithModeSync", False),
    # # ("sceFiosFHOpenSync", False),
    # # ("sceFiosFHPwriteSync", False),
    # # ("sceFiosStatSync", False),
    # # ("sceFiosDHReadSync", False),
    # # ("sceFiosFHTruncateSync", False),
    # # ("sceFiosDHOpen", False),
    # # ("sceFiosInitialize", False),
    # # ("sceFiosFHSeek", False),
    # ("sceGameLiveStreamingTerminate", False),
    # ("sceGameLiveStreamingGetCurrentStatus", False),
    # ("sceGameLiveStreamingGetProgramInfo", False),
    # ("sceGameLiveStreamingGetSocialFeedbackMessages", False),
    # ("sceGameLiveStreamingEnableLiveStreaming", False),
    # ("sceGameLiveStreamingInitialize", False),
    # ("sceGameLiveStreamingClearSocialFeedbackMessages", False),
    # ("sceGameLiveStreamingSetPresetSocialFeedbackCommandsDescription", False),
    # ("sceGameLiveStreamingSetPresetSocialFeedbackCommands", False),
    # ("sceGameLiveStreamingGetSocialFeedbackMessagesCount", False),
    # ("sceGnmInsertWaitFlipDone", False),
    # ("sceGnmMapComputeQueue", False),
    # ("sceGnmInsertPopMarker", False),
    # ("sceGnmSetEsShader", False),
    # ("sceGnmDrawInitDefaultHardwareState", False),
    # ("sceGnmSetCsShader", False),
    # ("sceGnmDeleteEqEvent", False),
    # # ("sceGnmDriverCaptureInProgress", False),
    # ("sceGnmSetGsShader", False),
    # ("sceGnmSetHsShader", False),
    # ("sceGnmInsertPushMarker", False),
    # ("sceGnmAddEqEvent", False),
    # ("sceGnmSetPsShader", False),
    # # ("sceGnmDingDong", False),
    # ("sceGnmSetVsShader", False),
    # ("sceGnmIsUserPaEnabled", False),
    # ("sceGnmDispatchInitDefaultHardwareState", False),
    # # ("sceGnmDebugHardwareStatus", False),
    # ("sceGnmSetLsShader", False),
    # ("sceGnmSubmitAndFlipCommandBuffers", False),
    # ("sceGnmSubmitDone", False),
    # ("sceHttpSetConnectTimeOut", False),
    # ("sceHttpGetStatusCode", False),
    # ("sceHttpCreateTemplate", False),
    # ("sceHttpGetLastErrno", False),
    # ("sceHttpSendRequest", False),
    # ("sceHttpDeleteTemplate", False),
    # ("sceHttpGetAuthEnabled", False),
    # ("sceHttpInit", False),
    # ("sceHttpCreateRequestWithURL", False),
    # ("sceHttpsLoadCert", False),
    # ("sceHttpAddRequestHeader", False),
    # ("sceHttpSetCookieSendCallback", False),
    # ("sceHttpTerm", False),
    # ("sceHttpSetCookieRecvCallback", False),
    # ("sceHttpCreateConnection", False),
    # ("sceHttpReadData", False),
    # ("sceHttpDeleteConnection", False),
    # ("sceHttpSetRequestContentLength", False),
    # ("sceHttpSetAutoRedirect", False),
    # ("sceHttpSetCookieEnabled", False),
    # ("sceHttpGetAllResponseHeaders", False),
    # ("sceHttpSetRedirectCallback", False),
    # ("sceHttpParseResponseHeader", False),
    # ("sceHttpsSetSslCallback", False),
    # ("sceHttpAbortRequest", False),
    # ("sceHttpGetCookieEnabled", False),
    # ("sceHttpAddCookie", False),
    # ("sceHttpGetAutoRedirect", False),
    # ("sceHttpSetAuthEnabled", False),
    # ("sceHttpDeleteRequest", False),
    # ("sceHttpCreateConnectionWithURL", False),
    # ("sceHttpCookieFlush", False),
    # ("sceHttpCreateRequest", False),
    # ("sceHttpSetSendTimeOut", False),
    # ("sceHttpSetRecvTimeOut", False),
    # ("sceHttpGetResponseContentLength", False),
    # ("sceHttpRemoveRequestHeader", False),
    # ("sceHttpsUnloadCert", False),
    # ("sceImeDialogGetStatus", False),
    # ("sceImeDialogInit", False),
    # ("sceImeDialogTerm", False),
    # ("sceImeDialogAbort", False),
    # ("sceImeDialogGetResult", False),
    # ("sceJpegDecDecode", False),
    # ("sceJpegDecDelete", False),
    # ("sceJpegDecCreate", False),
    # ("sceJpegDecParseHeader", False),
    # ("sceJpegDecQueryMemorySize", False),
    # ("sceMoveGetDeviceInfo", False),
    # ("sceMoveOpen", False),
    # ("sceMoveClose", False),
    # ("sceMoveReadStateRecent", False),
    # ("sceMoveInit", False),
    # ("sceMoveTrackerControllersUpdate", False),
    # ("sceMoveTrackerTerm", False),
    # ("sceMoveTrackerCameraUpdate", False),
    # ("sceMoveTrackerInit", False),
    # ("sceMoveTrackerGetState", False),
    # ("sceMoveTrackerGetWorkingMemorySize", False),
    # ("sceMsgDialogUpdateStatus", False),
    # ("sceMsgDialogOpen", False),
    # ("sceMsgDialogTerminate", False),
    # ("sceMsgDialogInitialize", False),
    # ("sceNetSetsockopt", False),
    # ("sceNetSocketClose", False),
    # ("sceNetInetPton", False),
    # ("sceNetInetNtop", False),
    # ("sceNetRecv", False),
    # ("sceNetResolverCreate", False),
    # ("sceNetErrnoLoc", False),
    # ("sceNetEpollDestroy", False),
    # ("sceNetPoolDestroy", False),
    # ("sceNetResolverStartNtoa", False),
    # ("sceNetInit", False),
    # ("sceNetConnect", False),
    # ("sceNetAccept", False),
    # ("sceNetSocket", False),
    # ("sceNetEpollCreate", False),
    # ("sceNetEpollControl", False),
    # ("sceNetBind", False),
    # ("sceNetSend", False),
    # ("sceNetTerm", False),
    # ("sceNetPoolCreate", False),
    # ("sceNetEpollWait", False),
    # ("sceNetHtons", False),
    # ("sceNetResolverDestroy", False),
    # ("sceNetListen", False),
    # ("sceNetCtlGetNatInfo", False),
    # ("sceNetCtlUnregisterCallback", False),
    # ("sceNetCtlRegisterCallback", False),
    # # ("sceNetCtlCheckCallback", False),
    # ("sceNetCtlGetInfo", False),
    # ("sceNetCtlGetState", False),
    # ("sceNpAuthCreateRequest", False),
    # ("sceNpAuthDeleteRequest", False),
    # ("sceNpAuthGetAuthorizationCode", False),
    # ("sceNpCommerceDialogInitialize", False),
    # ("sceNpCommerceShowPsStoreIcon", False),
    # ("sceNpCommerceDialogOpen", False),
    # ("sceNpCommerceDialogUpdateStatus", False),
    # ("sceNpCommerceHidePsStoreIcon", False),
    # ("sceNpCommerceDialogTerminate", False),
    # ("sceNpCommerceDialogGetResult", False),
    # ("sceNpCmpOnlineId", False),
    # # ("sceNpInGameMessageDeleteHandle", False),
    # # ("sceNpCheckCallback", False),
    # # ("sceNpSetContentRestriction", False),
    # # ("sceNpSetNpTitleId", False),
    # # ("sceNpInGameMessageInitialize", False),
    # # ("sceNpNotifyPlusFeature", False),
    # # ("sceNpGetAccountCountry", False),
    # # ("sceNpCreateRequest", False),
    # # ("sceNpInGameMessageSendData", False),
    # # ("sceNpAbortRequest", False),
    # # ("sceNpDeleteRequest", False),
    # # ("sceNpRegisterStateCallback", False),
    # # ("sceNpInGameMessagePrepare", False),
    # # ("sceNpGetOnlineId", False),
    # # ("sceNpInGameMessageTerminate", False),
    # # ("sceNpGetState", False),
    # # ("sceNpCreateAsyncRequest", False),
    # # ("sceNpGetParentalControlInfo", False),
    # # ("sceNpUnregisterStateCallback", False),
    # # ("sceNpGetNpId", False),
    # # ("sceNpCheckPlus", False),
    # # ("sceNpInGameMessageCreateHandle", False),
    # # ("sceNpPollAsync", False),
    # # ("sceNpSignalingActivateConnection", False),
    # # ("sceNpSignalingInitialize", False),
    # # ("sceNpSignalingCreateContext", False),
    # # ("sceNpSignalingGetConnectionInfo", False),
    # # ("sceNpSignalingTerminateConnection", False),
    # # ("sceNpSignalingGetConnectionStatus", False),
    # # ("sceNpSignalingDeleteContext", False),
    # # ("sceNpTrophyUnlockTrophy", False),
    # # ("sceNpTrophyDestroyContext", False),
    # # ("sceNpTrophyDestroyHandle", False),
    # # ("sceNpTrophyGetTrophyUnlockState", False),
    # # ("sceNpTrophyRegisterContext", False),
    # # ("sceNpTrophyCreateContext", False),
    # # ("sceNpTrophyCreateHandle", False),
    # # ("sceNpLookupCreateTitleCtx", False),
    # # ("sceNpBandwidthTestGetStatus", False),
    # # ("sceNpLookupNpId", False),
    # # ("sceNpLookupCreateRequest", False),
    # # ("sceNpBandwidthTestInitStart", False),
    # # ("sceNpLookupDeleteTitleCtx", False),
    # # ("sceNpBandwidthTestShutdown", False),
    # # ("sceNpLookupDeleteRequest", False),
    # # ("sceNpWebApiGetErrorCode", False),
    # # ("sceNpWebApiDeleteHandle", False),
    # # ("sceNpWebApiCreateHandle", False),
    # # ("sceNpWebApiReadData", False),
    # # ("sceNpWebApiInitialize", False),
    # # ("sceNpWebApiAbortRequest", False),
    # # ("sceNpWebApiDeleteServicePushEventFilter", False),
    # # ("sceNpWebApiRegisterPushEventCallback", False),
    # # ("sceNpWebApiDeleteContext", False),
    # # ("sceNpWebApiTerminate", False),
    # # ("sceNpWebApiGetHttpStatusCode", False),
    # # ("sceNpWebApiRegisterServicePushEventCallback", False),
    # # ("sceNpWebApiSendRequest", False),
    # # ("sceNpWebApiDeleteRequest", False),
    # # ("sceNpWebApiUtilityParseNpId", False),
    # # ("sceNpWebApiUnregisterPushEventCallback", False),
    # # ("sceNpWebApiCreateRequest", False),
    # # ("sceNpWebApiCreateServicePushEventFilter", False),
    # # ("sceNpWebApiCreateContext", False),
    # # ("sceNpWebApiCreatePushEventFilter", False),

    # # todo: why does scePad break everything?
    # # ("scePadClose", False),
    # # ("scePadSetLightBar", False),
    # # ("scePadReadState", False),
    # # ("scePadGetControllerInformation", False),
    # # ("scePadInit", False),
    # # ("scePadOpen", False),
    # # ("scePadSetVibration", False),

    # ("scePlayGoGetProgress", False),
    # ("scePlayGoSetInstallSpeed", False),
    # ("scePlayGoOpen", False),
    # ("scePlayGoGetToDoList", False),
    # ("scePlayGoClose", False),
    # ("scePlayGoGetInstallSpeed", False),
    # ("scePlayGoInitialize", False),
    # ("scePlayGoGetLocus", False),

    # # ("recv", False),
    # # ("sem_post", False),
    # # ("bind", False),
    # # ("socket", False),
    # # ("shutdown", False),
    # # ("connect", False),
    # # ("pthread_setschedparam", False),
    # # ("sem_wait", False),
    # # ("sem_destroy", False),
    # # ("setsockopt", False),
    # # ("send", False),
    # # ("clock_gettime", False),
    # # ("recvfrom", False),
    # # ("sendto", False),
    # # ("sem_init", False),
    # # ("sem_timedwait", False),
    # # ("sceRtcGetCurrentTick", True),
    # # ("sceRtcGetTick", False),
    # # ("sceRtcGetDayOfWeek", False),
    # # ("sceRtcConvertUtcToLocalTime", False),
    # # ("sceRtcGetCurrentClockLocalTime", False),
    # # ("sceRtcSetTime_t", False),
    # # ("sceRtcSetTick", False),
    # # ("sceRtcGetCurrentNetworkTick", False),
    # ("sceSaveDataMount", False),
    # ("sceSaveDataSetParam", False),
    # ("sceSaveDataUmount", False),
    # ("sceSaveDataDelete", False),
    # ("sceSaveDataInitialize", False),
    # ("sceSaveDataSaveIcon", False),
    # ("sceSaveDataDirNameSearch", False),
    # ("sceSaveDataTerminate", False),
    # ("sceSaveDataDialogOpen", False),
    # ("sceSaveDataDialogUpdateStatus", False),
    # ("sceSaveDataDialogTerminate", False),
    # ("sceSaveDataDialogIsReadyToDisplay", False),
    # ("sceSaveDataDialogClose", False),
    # ("sceSaveDataDialogProgressBarSetValue", False),
    # ("sceSaveDataDialogInitialize", False),
    # ("sceSaveDataDialogGetResult", False),
    # ("sceSslTerm", False),
    # ("sceSslInit", False),
    # ("sceSysmoduleIsLoaded", False),
    # ("sceSysmoduleLoadModule", False),
    # ("sceSystemServiceGetDisplaySafeAreaInfo", False),
    # ("sceSystemServiceReceiveEvent", False),
    # ("sceSystemServiceHideSplashScreen", False),
    # ("sceSystemServiceParamGetInt", False),
    # ("sceSystemServiceGetStatus", False),

    # # TODO: Why does user service crash ugh
    # # ("sceUserServiceGetUserName", False),
    # # ("sceUserServiceGetInitialUser", False),
    # # ("sceUserServiceTerminate", False),
    # # ("sceUserServiceGetLoginUserIdList", False),
    # # ("sceUserServiceInitialize", False),
    # # ("sceUserServiceGetEvent", False),

    # ("sceVideoOutSetFlipRate", False),
    # ("sceVideoOutColorSettingsSetGamma_", False),
    # ("sceVideoOutAddFlipEvent", False),
    # ("sceVideoOutSetWindowModeMargins", False),
    # ("sceVideoOutOpen", False),
    # ("sceVideoOutSetBufferAttribute", False),
    # ("sceVideoOutAdjustColor_", False),
    # ("sceVideoOutRegisterBuffers", False),
    # # ("scePthreadSetspecific", True),
    # # ("sceKernelReadTsc", True),
    # # # ("scePthreadAttrSetdetachstate", True),
    # # ("sceKernelMkdir", True),
    # # ("sceKernelPollSema", True),
    # # ("sceKernelCreateSema", True),
    # # # ("sceKernelOpen", True),
    # # # ("sceKernelGetTscFrequency", True),
    # # ("sceKernelUsleep", True),
    # # ("sceKernelGetEventFilter", True),
    # # # ("scePthreadMutexDestroy", True),
    # # # ("scePthreadCondInit", True),
    # # # ("scePthreadExit", True),
    # # # ("scePthreadAttrSetaffinity", True),
    # # # ("scePthreadAttrSetschedpolicy", True),
    # # # ("sceKernelAddUserEvent", True),
    # # # ("sceKernelSignalSema", True),
    # # # ("scePthreadDetach", True),
    # # # ("sceKernelAddTimerEvent", True),
    # # # ("scePthreadAttrDestroy", True),
    # # # # ("scePthreadCreate", True),
    # # # ("__error", False),
    # # # # ("scePthreadMutexLock", True),
    # # # ("sceKernelCreateEventFlag", True),
    # # # ("scePthreadAttrSetstack", True),
    # # # ("sceKernelRead", True),
    # # # ("sceKernelCreateEqueue", True),
    # # # ("scePthreadAttrSetschedparam", True),
    # # # ("scePthreadMutexattrInit", True),
    # # # ("sceKernelSetEventFlag", True),
    # # # ("scePthreadMutexTimedlock", True),
    # # # ("scePthreadCondBroadcast", True),
    # # # ("sceKernelWaitEventFlag", True),
    # # # ("sceKernelMapNamedDirectMemory", True),
    # # # ("__stack_chk_fail", False),
    # # # ("scePthreadGetschedparam", True),
    # # # # ("sceKernelClockGettime", True),
    # # # ("sceKernelDeleteSema", True),
    # # # # ("scePthreadYield", True),
    # # # ("sceKernelClose", True),
    # # # ("scePthreadAttrSetstacksize", True),
    # # # ("scePthreadSetprio", True),
    # # # ("sceKernelDeleteTimerEvent", True),
    # # # ("sceKernelWaitSema", True),
    # # # # ("scePthreadSelf", True),
    # # # ("scePthreadSetaffinity", True),
    # # # ("scePthreadMutexInit", True),
    # # # ("scePthreadAttrSetinheritsched", True),
    # # # ("sceKernelGettimeofday", True),
    # # # ("scePthreadGetspecific", True),
    # # # ("sceKernelWaitEqueue", True),
    # # # ("scePthreadCondDestroy", True),
    # # # ("scePthreadKeyCreate", True),
    # # # ("scePthreadMutexattrSettype", True),
    # # # ("sceKernelDeleteEqueue", True),
    # # # ("sceKernelFstat", True),
    # # # ("sceKernelGetEventId", True),
    # # # ("scePthreadAttrInit", True),
    # # # ("scePthreadSetschedparam", True),
    # # # ("sceKernelLseek", True),
    # # # ("scePthreadJoin", True),
    # # # ("sceKernelGetDirectMemorySize", True),
    # # # ("sceKernelAllocateDirectMemory", True),
    # # # ("sceKernelVirtualQuery", True),
    # # # ("scePthreadMutexattrDestroy", True),
    # # # # ("scePthreadMutexUnlock", True),
    # # # # ("scePthreadMutexTrylock", True),
    # # # # ("__tls_get_addr", False),
    # # # # ("sceKernelLoadStartModule", True),
]

template = r"""
#include "args.h"

#define VM_PROT_NONE ((vm_prot_t)0x00)
#define VM_PROT_READ ((vm_prot_t)0x01)
#define VM_PROT_WRITE ((vm_prot_t)0x02)
#define VM_PROT_EXECUTE ((vm_prot_t)0x04)
#define VM_PROT_COPY ((vm_prot_t)0x08) /* copy-on-read */

#define VM_PROT_ALL (VM_PROT_READ | VM_PROT_WRITE | VM_PROT_EXECUTE)

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wreturn-type"

% for (label_id, (name, address)) in enumerate(imports):
void *original_${name} = 0x0;

void ${name}_start_logger()
{
    emit_span_start(${label_id});
}

void ${name}_end_logger()
{
    emit_span_end();
}

__attribute__((naked)) void *${name}_hook()
{
    SAVE_ARGS_STATE();
    asm volatile("call ${name}_start_logger\n\t");
    RESTORE_ARGS_STATE();

    asm volatile(
        // backup return address
        // use r10 as a scratch register. it is a caller saved register and not
        // an argument register.
        "pop %%r10\n\t"
        
        // store the value in thread local storage slot. there are no registers
        // which we can use in this context which will both
        // 1. not need to be backed up
        // 2. saved across call boundary
        // callee saved registers are saved across the call boundary but we
        // need to backup for our caller.
        // caller saved registers will not be saved across the call boundary
        // but do not need to be backed up prior to usage
        "movq %%r10, %%fs:-16\n\t"
        :
        :
        :
    );
    
    asm volatile(
        // execute original function
        "mov %0, %%rax\n\t"
        "call *%%rax\n\t"
        : : "m"(original_${name})
    );

    asm volatile(
        // restore return address
        "movq %%fs:-16, %%r10\n\t"
        "push %%r10\n\t"

        // backup rax (also happens to align stack for call)
        "push %%rax\n\t"

        // call end logger
        "call ${name}_end_logger\n\t"

        // restore rax (ignore the return value of end logger)
        "pop %%rax\n\t"

        "ret\n\t"
        :
        :
        : "r12", "rax"
    );
}
% endfor

#pragma GCC diagnostic pop

void register_hooks()
{
% for (name, address) in imports:
    {
        uint64_t *function_ptr = (uint64_t *)${address};
        sceKernelMprotect((void *)function_ptr, sizeof(uint64_t), VM_PROT_ALL);
        original_${name} = (void *)*function_ptr;
        *function_ptr = (uint64_t)${name}_hook;
    }
% endfor
}
"""

if __name__ == '__main__':
    generated = Template(template).render(imports=imports)

    with open('trampolines.h', 'w') as file:
        file.write(generated)
