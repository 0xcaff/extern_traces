#include <stdalign.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>

#include "plugin_common.h"

#define SAVE_ARGS_STATE()               \
    asm volatile(                       \
        "push %rdi\n\t"                 \
        "push %rsi\n\t"                 \
        "push %rdx\n\t"                 \
        "push %rcx\n\t"                 \
        "push %r8\n\t"                  \
        "push %r9\n\t"                  \
        "movdqu %xmm0, -0x10(%rsp)\n\t" \
        "movdqu %xmm1, -0x20(%rsp)\n\t" \
        "movdqu %xmm2, -0x30(%rsp)\n\t" \
        "movdqu %xmm3, -0x40(%rsp)\n\t" \
        "movdqu %xmm4, -0x50(%rsp)\n\t" \
        "movdqu %xmm5, -0x60(%rsp)\n\t" \
        "movdqu %xmm6, -0x70(%rsp)\n\t" \
        "movdqu %xmm7, -0x80(%rsp)\n\t" \
        "sub $0x80, %rsp\n\t")

#define RESTORE_ARGS_STATE()            \
    asm volatile(                       \
        "add $0x80, %rsp\n\t"           \
        "movdqu -0x10(%rsp), %xmm0\n\t" \
        "movdqu -0x20(%rsp), %xmm1\n\t" \
        "movdqu -0x30(%rsp), %xmm2\n\t" \
        "movdqu -0x40(%rsp), %xmm3\n\t" \
        "movdqu -0x50(%rsp), %xmm4\n\t" \
        "movdqu -0x60(%rsp), %xmm5\n\t" \
        "movdqu -0x70(%rsp), %xmm6\n\t" \
        "movdqu -0x80(%rsp), %xmm7\n\t" \
        "pop %r9\n\t"                   \
        "pop %r8\n\t"                   \
        "pop %rcx\n\t"                  \
        "pop %rdx\n\t"                  \
        "pop %rsi\n\t"                  \
        "pop %rdi\n\t")

attr_public const char *g_pluginName = "extern_traces";
attr_public const char *g_pluginDesc = "Collects traces for external calls";
attr_public const char *g_pluginAuth = "0xcaff";
attr_public uint32_t g_pluginVersion = 0x00000100; // 1.00

static int file_descriptor;

int lazy_file()
{
    if (file_descriptor)
    {
        return file_descriptor;
    }

    final_printf("opening file...\n");
    int fd = sceKernelOpen(
        "/data/extern.log",
        0x0400 | 0x0200,
        0666);
    if (fd < 0)
    {
        final_printf("failed to open file /data/extern.log\n");
        return 0;
    }
    final_printf("opened file!\n");

    file_descriptor = fd;
    return fd;
}

void extern_logf(const char *str)
{
    int fd = lazy_file();
    int len = strlen(str);
    sceKernelWrite(fd, str, len);
    sceKernelSync();
}

#pragma GCC diagnostic push
#pragma GCC diagnostic ignored "-Wreturn-type"

#include "trampolines.h"

#pragma GCC diagnostic pop

s32 attr_module_hidden module_start(s64 argc, const void *args)
{
    final_printf("[GoldHEN] %s Plugin Started.\n", g_pluginName);
    final_printf("[GoldHEN] <%s\\Ver.0x%08x> %s\n", g_pluginName, g_pluginVersion, __func__);
    final_printf("[GoldHEN] Plugin Author(s): %s\n", g_pluginAuth);

    // HOOK32(sceAppContentAppParamGetInt);
    // HOOK32(sceAppContentInitialize);
    // HOOK32(sceAppContentTemporaryDataGetAvailableSpaceKb);
    // HOOK32(sceAppContentGetEntitlementKey);
    // HOOK32(sceAppContentTemporaryDataFormat);
    // HOOK32(sceAppContentTemporaryDataMount2);
    // HOOK32(sceAppContentGetAddcontInfoList);
    // HOOK32(sceAudioInOpen);
    // HOOK32(sceAudioInInput);
    HOOK32(sceAudioOutInit);
    // HOOK32(sceAudioOutOutput);
    // HOOK32(sceAudioOutSetVolume);
    // HOOK32(sceAudioOutOpen);
    // HOOK32(sceAudioOutClose);
    // HOOK32(sceAvPlayerPause);
    // HOOK32(sceAvPlayerStart);
    // HOOK32(sceAvPlayerGetVideoDataEx);
    // HOOK32(sceAvPlayerAddSource);
    // HOOK32(sceAvPlayerClose);
    // HOOK32(sceAvPlayerEnableStream);
    // HOOK32(sceAvPlayerSetLooping);
    // HOOK32(sceAvPlayerIsActive);
    // HOOK32(sceAvPlayerGetAudioData);
    // HOOK32(sceAvPlayerJumpToTime);
    // HOOK32(sceAvPlayerInit);
    // HOOK32(sceAvPlayerGetStreamInfo);
    // HOOK32(sceAvPlayerStreamCount);
    // HOOK32(sceAvPlayerResume);
    // HOOK32(sceAvPlayerCurrentTime);
    // HOOK32(sceCameraStop);
    // HOOK32(sceCameraStart);
    // HOOK32(sceCameraOpen);
    // HOOK32(sceCameraClose);
    // HOOK32(sceCameraSetConfig);
    // HOOK32(sceCameraGetConfig);
    // HOOK32(sceCameraSetAttribute);
    // HOOK32(sceCameraGetFrameData);
    // HOOK32(sceCameraIsAttached);
    // HOOK32(sceCommonDialogInitialize);
    // HOOK32(sceCompanionHttpdStop);
    // HOOK32(sceCompanionHttpdAddHeader);
    // HOOK32(sceCompanionHttpdRegisterRequestCallback);
    // HOOK32(sceCompanionHttpdStart);
    // HOOK32(sceCompanionHttpdOptParamInitialize);
    // HOOK32(sceCompanionHttpdSetStatus);
    // HOOK32(sceCompanionHttpdInitialize);
    // HOOK32(sceContentSearchTerm);
    // HOOK32(sceContentSearchGetMyApplicationIndex);
    // HOOK32(sceContentSearchGetContentLastUpdateId);
    // HOOK32(sceContentSearchSearchContent);
    // HOOK32(sceContentSearchInit);
    // HOOK32(sceErrorDialogTerminate);
    // HOOK32(sceErrorDialogInitialize);
    // HOOK32(sceErrorDialogOpen);
    // HOOK32(sceErrorDialogUpdateStatus);
    // HOOK32(sceFiberReturnToThread);
    // HOOK32(sceFiberFinalize);
    // HOOK32(sceFiberRun);
    // HOOK32(sceFiberOptParamInitialize);
    // HOOK32(_sceFiberInitializeImpl);
    // HOOK32(sceFiosOpGetActualCount);
    // HOOK32(sceFiosDHCloseSync);
    // HOOK32(sceFiosOpDelete);
    // HOOK32(sceFiosFHCloseSync);
    // HOOK32(sceFiosFHReadSync);
    // HOOK32(sceFiosFHSyncSync);
    // HOOK32(sceFiosFHGetSize);
    // HOOK32(sceFiosRenameSync);
    // HOOK32(sceFiosDHOpenSync);
    // HOOK32(sceFiosFHWriteSync);
    // HOOK32(sceFiosDeleteSync);
    // HOOK32(sceFiosFileExistsSync);
    // HOOK32(sceFiosDirectoryExistsSync);
    // HOOK32(sceFiosOpWait);
    // HOOK32(sceFiosDirectoryCreateWithModeSync);
    // HOOK32(sceFiosFHOpenSync);
    // HOOK32(sceFiosFHPwriteSync);
    // HOOK32(sceFiosStatSync);
    // HOOK32(sceFiosDHReadSync);
    // HOOK32(sceFiosFHTruncateSync);
    // HOOK32(sceFiosDHOpen);
    // HOOK32(sceFiosInitialize);
    // HOOK32(sceFiosFHSeek);
    // HOOK32(sceGameLiveStreamingTerminate);
    // HOOK32(sceGameLiveStreamingGetCurrentStatus);
    // HOOK32(sceGameLiveStreamingGetProgramInfo);
    // HOOK32(sceGameLiveStreamingGetSocialFeedbackMessages);
    // HOOK32(sceGameLiveStreamingEnableLiveStreaming);
    // HOOK32(sceGameLiveStreamingInitialize);
    // HOOK32(sceGameLiveStreamingClearSocialFeedbackMessages);
    // HOOK32(sceGameLiveStreamingSetPresetSocialFeedbackCommandsDescription);
    // HOOK32(sceGameLiveStreamingSetPresetSocialFeedbackCommands);
    // HOOK32(sceGameLiveStreamingGetSocialFeedbackMessagesCount);
    // HOOK32(sceGnmInsertWaitFlipDone);
    // HOOK32(sceGnmMapComputeQueue);
    // HOOK32(sceGnmInsertPopMarker);
    // HOOK32(sceGnmSetEsShader);
    // HOOK32(sceGnmDrawInitDefaultHardwareState);
    // HOOK32(sceGnmSetCsShader);
    // HOOK32(sceGnmDeleteEqEvent);
    // HOOK32(sceGnmDriverCaptureInProgress);
    // HOOK32(sceGnmSetGsShader);
    // HOOK32(sceGnmSetHsShader);
    // HOOK32(sceGnmInsertPushMarker);
    // HOOK32(sceGnmAddEqEvent);
    // HOOK32(sceGnmSetPsShader);
    // // HOOK32(sceGnmDingDong);
    // HOOK32(sceGnmSetVsShader);
    // HOOK32(sceGnmIsUserPaEnabled);
    // HOOK32(sceGnmDispatchInitDefaultHardwareState);
    // HOOK32(sceGnmDebugHardwareStatus);
    // HOOK32(sceGnmSetLsShader);
    // HOOK32(sceGnmSubmitAndFlipCommandBuffers);
    // HOOK32(sceGnmSubmitDone);
    // HOOK32(sceHttpSetConnectTimeOut);
    // HOOK32(sceHttpGetStatusCode);
    // HOOK32(sceHttpCreateTemplate);
    // HOOK32(sceHttpGetLastErrno);
    // HOOK32(sceHttpSendRequest);
    // HOOK32(sceHttpDeleteTemplate);
    // HOOK32(sceHttpGetAuthEnabled);
    // HOOK32(sceHttpInit);
    // HOOK32(sceHttpCreateRequestWithURL);
    // HOOK32(sceHttpsLoadCert);
    // HOOK32(sceHttpAddRequestHeader);
    // HOOK32(sceHttpSetCookieSendCallback);
    // HOOK32(sceHttpTerm);
    // HOOK32(sceHttpSetCookieRecvCallback);
    // HOOK32(sceHttpCreateConnection);
    // HOOK32(sceHttpReadData);
    // HOOK32(sceHttpDeleteConnection);
    // HOOK32(sceHttpSetRequestContentLength);
    // HOOK32(sceHttpSetAutoRedirect);
    // HOOK32(sceHttpSetCookieEnabled);
    // HOOK32(sceHttpGetAllResponseHeaders);
    // HOOK32(sceHttpSetRedirectCallback);
    // HOOK32(sceHttpParseResponseHeader);
    // HOOK32(sceHttpsSetSslCallback);
    // HOOK32(sceHttpAbortRequest);
    // HOOK32(sceHttpGetCookieEnabled);
    // HOOK32(sceHttpAddCookie);
    // HOOK32(sceHttpGetAutoRedirect);
    // HOOK32(sceHttpSetAuthEnabled);
    // HOOK32(sceHttpDeleteRequest);
    // HOOK32(sceHttpCreateConnectionWithURL);
    // HOOK32(sceHttpCookieFlush);
    // HOOK32(sceHttpCreateRequest);
    // HOOK32(sceHttpSetSendTimeOut);
    // HOOK32(sceHttpSetRecvTimeOut);
    // HOOK32(sceHttpGetResponseContentLength);
    // HOOK32(sceHttpRemoveRequestHeader);
    // HOOK32(sceHttpsUnloadCert);
    // HOOK32(sceImeDialogGetStatus);
    // HOOK32(sceImeDialogInit);
    // HOOK32(sceImeDialogTerm);
    // HOOK32(sceImeDialogAbort);
    // HOOK32(sceImeDialogGetResult);
    // HOOK32(sceJpegDecDecode);
    // HOOK32(sceJpegDecDelete);
    // HOOK32(sceJpegDecCreate);
    // HOOK32(sceJpegDecParseHeader);
    // HOOK32(sceJpegDecQueryMemorySize);
    // HOOK32(sceMoveGetDeviceInfo);
    // HOOK32(sceMoveOpen);
    // HOOK32(sceMoveClose);
    // HOOK32(sceMoveReadStateRecent);
    // HOOK32(sceMoveInit);
    // HOOK32(sceMoveTrackerControllersUpdate);
    // HOOK32(sceMoveTrackerTerm);
    // HOOK32(sceMoveTrackerCameraUpdate);
    // HOOK32(sceMoveTrackerInit);
    // HOOK32(sceMoveTrackerGetState);
    // HOOK32(sceMoveTrackerGetWorkingMemorySize);
    // HOOK32(sceMsgDialogUpdateStatus);
    // HOOK32(sceMsgDialogOpen);
    // HOOK32(sceMsgDialogTerminate);
    // HOOK32(sceMsgDialogInitialize);
    // HOOK32(sceNetSetsockopt);
    // HOOK32(sceNetSocketClose);
    // HOOK32(sceNetInetPton);
    // HOOK32(sceNetInetNtop);
    // HOOK32(sceNetRecv);
    // HOOK32(sceNetResolverCreate);
    // HOOK32(sceNetErrnoLoc);
    // HOOK32(sceNetEpollDestroy);
    // HOOK32(sceNetPoolDestroy);
    // HOOK32(sceNetResolverStartNtoa);
    // HOOK32(sceNetInit);
    // HOOK32(sceNetConnect);
    // HOOK32(sceNetAccept);
    // HOOK32(sceNetSocket);
    // HOOK32(sceNetEpollCreate);
    // HOOK32(sceNetEpollControl);
    // HOOK32(sceNetBind);
    // HOOK32(sceNetSend);
    // HOOK32(sceNetTerm);
    // HOOK32(sceNetPoolCreate);
    // HOOK32(sceNetEpollWait);
    // HOOK32(sceNetHtons);
    // HOOK32(sceNetResolverDestroy);
    // HOOK32(sceNetListen);
    // HOOK32(sceNetCtlGetNatInfo);
    // HOOK32(sceNetCtlUnregisterCallback);
    // HOOK32(sceNetCtlRegisterCallback);
    // HOOK32(sceNetCtlCheckCallback);
    // HOOK32(sceNetCtlGetInfo);
    // HOOK32(sceNetCtlGetState);
    // HOOK32(sceNpAuthCreateRequest);
    // HOOK32(sceNpAuthDeleteRequest);
    // HOOK32(sceNpAuthGetAuthorizationCode);
    // HOOK32(sceNpCommerceDialogInitialize);
    // HOOK32(sceNpCommerceShowPsStoreIcon);
    // HOOK32(sceNpCommerceDialogOpen);
    // HOOK32(sceNpCommerceDialogUpdateStatus);
    // HOOK32(sceNpCommerceHidePsStoreIcon);
    // HOOK32(sceNpCommerceDialogTerminate);
    // HOOK32(sceNpCommerceDialogGetResult);
    // HOOK32(sceNpCmpOnlineId);
    // // HOOK32(sceNpInGameMessageDeleteHandle);
    // // HOOK32(sceNpCheckNpAvailability);
    // // HOOK32(sceNpCheckCallback);
    // // HOOK32(sceNpSetContentRestriction)
    // // HOOK32(sceNpSetNpTitleId);
    // // HOOK32(sceNpInGameMessageInitialize);
    // // HOOK32(sceNpNotifyPlusFeature);
    // // HOOK32(sceNpGetAccountCountry);
    // // HOOK32(sceNpCreateRequest);
    // // HOOK32(sceNpInGameMessageSendData);
    // // HOOK32(sceNpAbortRequest);
    // // HOOK32(sceNpDeleteRequest);
    // // HOOK32(sceNpRegisterStateCallback);
    // // HOOK32(sceNpInGameMessagePrepare);
    // // HOOK32(sceNpGetOnlineId);
    // // HOOK32(sceNpInGameMessageTerminate);
    // // HOOK32(sceNpGetState);
    // // HOOK32(sceNpCreateAsyncRequest);
    // // HOOK32(sceNpGetParentalControlInfo);
    // // HOOK32(sceNpUnregisterStateCallback);
    // // HOOK32(sceNpGetNpId);
    // // HOOK32(sceNpCheckPlus);
    // // HOOK32(sceNpInGameMessageCreateHandle);
    // // HOOK32(sceNpPollAsync);
    // // HOOK32(sceNpSignalingActivateConnection);
    // // HOOK32(sceNpSignalingInitialize);
    // // HOOK32(sceNpSignalingCreateContext);
    // // HOOK32(sceNpSignalingGetConnectionInfo);
    // // HOOK32(sceNpSignalingTerminateConnection);
    // // HOOK32(sceNpSignalingGetConnectionStatus);
    // // HOOK32(sceNpSignalingDeleteContext);
    // // HOOK32(sceNpTrophyUnlockTrophy);
    // // HOOK32(sceNpTrophyDestroyContext);
    // // HOOK32(sceNpTrophyDestroyHandle);
    // // HOOK32(sceNpTrophyGetTrophyUnlockState);
    // // HOOK32(sceNpTrophyRegisterContext);
    // // HOOK32(sceNpTrophyCreateContext);
    // // HOOK32(sceNpTrophyCreateHandle);
    // // HOOK32(sceNpLookupCreateTitleCtx);
    // // HOOK32(sceNpBandwidthTestGetStatus);
    // // HOOK32(sceNpLookupNpId);
    // // HOOK32(sceNpLookupCreateRequest);
    // // HOOK32(sceNpBandwidthTestInitStart);
    // // HOOK32(sceNpLookupDeleteTitleCtx);
    // // HOOK32(sceNpBandwidthTestShutdown);
    // // HOOK32(sceNpLookupDeleteRequest);
    // // HOOK32(sceNpWebApiGetErrorCode);
    // // HOOK32(sceNpWebApiDeleteHandle);
    // // HOOK32(sceNpWebApiCreateHandle);
    // // HOOK32(sceNpWebApiReadData);
    // // HOOK32(sceNpWebApiInitialize);
    // // HOOK32(sceNpWebApiAbortRequest);
    // // HOOK32(sceNpWebApiDeleteServicePushEventFilter);
    // // HOOK32(sceNpWebApiRegisterPushEventCallback);
    // // HOOK32(sceNpWebApiDeleteContext);
    // // HOOK32(sceNpWebApiTerminate);
    // // HOOK32(sceNpWebApiGetHttpStatusCode);
    // // HOOK32(sceNpWebApiRegisterServicePushEventCallback);
    // // HOOK32(sceNpWebApiSendRequest);
    // // HOOK32(sceNpWebApiDeleteRequest);
    // // HOOK32(sceNpWebApiUtilityParseNpId);
    // // HOOK32(sceNpWebApiUnregisterPushEventCallback);
    // // HOOK32(sceNpWebApiCreateRequest);
    // // HOOK32(sceNpWebApiCreateServicePushEventFilter);
    // // HOOK32(sceNpWebApiCreateContext);
    // // HOOK32(sceNpWebApiCreatePushEventFilter);
    // HOOK32(scePadClose);
    // HOOK32(scePadSetLightBar);
    // HOOK32(scePadReadState);
    // HOOK32(scePadGetControllerInformation);
    // HOOK32(scePadInit);
    // HOOK32(scePadOpen);
    // HOOK32(scePadSetVibration);
    // HOOK32(scePlayGoGetProgress);
    // HOOK32(scePlayGoSetInstallSpeed);
    // HOOK32(scePlayGoOpen);
    // HOOK32(scePlayGoGetToDoList);
    // HOOK32(scePlayGoClose);
    // HOOK32(scePlayGoGetInstallSpeed);
    // HOOK32(scePlayGoInitialize);
    // HOOK32(scePlayGoGetLocus);
    // // HOOK32(recv);
    // // HOOK32(sem_post);
    // // HOOK32(bind);
    // // HOOK32(socket);
    // // HOOK32(shutdown);
    // // HOOK32(connect);
    // // HOOK32(pthread_setschedparam);
    // // HOOK32(sem_wait);
    // // HOOK32(sem_destroy);
    // // HOOK32(setsockopt);
    // // HOOK32(send);
    // // HOOK32(clock_gettime);
    // // HOOK32(recvfrom);
    // // HOOK32(sendto);
    // // HOOK32(sem_init);
    // // HOOK32(sem_timedwait);
    // HOOK32(sceRtcGetCurrentTick);
    // HOOK32(sceRtcGetTick);
    // HOOK32(sceRtcGetDayOfWeek);
    // HOOK32(sceRtcConvertUtcToLocalTime);
    // HOOK32(sceRtcGetCurrentClockLocalTime);
    // HOOK32(sceRtcSetTime_t);
    // HOOK32(sceRtcSetTick);
    // HOOK32(sceRtcGetCurrentNetworkTick);
    // HOOK32(sceSaveDataMount);
    // HOOK32(sceSaveDataSetParam);
    // HOOK32(sceSaveDataUmount);
    // HOOK32(sceSaveDataDelete);
    // HOOK32(sceSaveDataInitialize);
    // HOOK32(sceSaveDataSaveIcon);
    // HOOK32(sceSaveDataDirNameSearch);
    // HOOK32(sceSaveDataTerminate);
    // HOOK32(sceSaveDataDialogOpen);
    // HOOK32(sceSaveDataDialogUpdateStatus);
    // HOOK32(sceSaveDataDialogTerminate);
    // HOOK32(sceSaveDataDialogIsReadyToDisplay);
    // HOOK32(sceSaveDataDialogClose);
    // HOOK32(sceSaveDataDialogProgressBarSetValue);
    // HOOK32(sceSaveDataDialogInitialize);
    // HOOK32(sceSaveDataDialogGetResult);
    // HOOK32(sceSslTerm);
    // HOOK32(sceSslInit);
    // HOOK32(sceSysmoduleIsLoaded);
    // HOOK32(sceSysmoduleLoadModule);
    // HOOK32(sceSystemServiceGetDisplaySafeAreaInfo);
    // HOOK32(sceSystemServiceReceiveEvent);
    // HOOK32(sceSystemServiceHideSplashScreen);
    // HOOK32(sceSystemServiceParamGetInt);
    // HOOK32(sceSystemServiceGetStatus);
    // HOOK32(sceUserServiceGetUserName);
    // HOOK32(sceUserServiceGetInitialUser);
    // HOOK32(sceUserServiceTerminate);
    // HOOK32(sceUserServiceGetLoginUserIdList);
    // HOOK32(sceUserServiceInitialize);
    // HOOK32(sceUserServiceGetEvent);
    // HOOK32(sceVideoOutSetFlipRate);
    // HOOK32(sceVideoOutColorSettingsSetGamma_);
    // HOOK32(sceVideoOutAddFlipEvent);
    // HOOK32(sceVideoOutSetWindowModeMargins);
    // HOOK32(sceVideoOutOpen);
    // HOOK32(sceVideoOutSetBufferAttribute);
    // HOOK32(sceVideoOutAdjustColor_);
    // HOOK32(sceVideoOutRegisterBuffers);
    // HOOK32(scePthreadSetspecific);
    // HOOK32(sceKernelReadTsc);
    // HOOK32(scePthreadAttrSetdetachstate);
    // HOOK32(sceKernelMkdir);
    // HOOK32(sceKernelPollSema);
    // HOOK32(sceKernelCreateSema);
    // HOOK32(sceKernelOpen);
    // HOOK32(sceKernelGetTscFrequency);
    // HOOK32(sceKernelUsleep);
    // HOOK32(sceKernelGetEventFilter);
    // HOOK32(scePthreadMutexDestroy);
    // HOOK32(scePthreadCondInit);
    // HOOK32(scePthreadExit);
    // HOOK32(scePthreadAttrSetaffinity);
    // HOOK32(scePthreadAttrSetschedpolicy);
    // HOOK32(sceKernelAddUserEvent);
    // HOOK32(sceKernelSignalSema);
    // HOOK32(scePthreadDetach);
    // HOOK32(sceKernelAddTimerEvent);
    // HOOK32(scePthreadAttrDestroy);
    // HOOK32(scePthreadCreate);
    // HOOK32(__error);
    // // HOOK32(scePthreadMutexLock);
    // HOOK32(sceKernelCreateEventFlag);
    // HOOK32(scePthreadAttrSetstack);
    // HOOK32(sceKernelRead);
    // HOOK32(sceKernelCreateEqueue);
    // HOOK32(scePthreadAttrSetschedparam);
    // HOOK32(scePthreadMutexattrInit);
    // HOOK32(sceKernelSetEventFlag);
    // HOOK32(scePthreadMutexTimedlock);
    // HOOK32(scePthreadCondBroadcast);
    // HOOK32(sceKernelWaitEventFlag);
    // HOOK32(sceKernelMapNamedDirectMemory);
    // HOOK32(__stack_chk_fail);
    // HOOK32(scePthreadGetschedparam);
    // HOOK32(sceKernelClockGettime);
    // HOOK32(sceKernelDeleteSema);
    // // HOOK32(scePthreadYield);
    // HOOK32(sceKernelClose);
    // HOOK32(scePthreadAttrSetstacksize);
    // HOOK32(scePthreadSetprio);
    // HOOK32(sceKernelDeleteTimerEvent);
    // HOOK32(sceKernelWaitSema);
    // // HOOK32(scePthreadSelf);
    // HOOK32(scePthreadSetaffinity);
    // HOOK32(scePthreadMutexInit);
    // HOOK32(scePthreadAttrSetinheritsched);
    // HOOK32(sceKernelGettimeofday);
    // HOOK32(scePthreadGetspecific);
    // HOOK32(sceKernelWaitEqueue);
    // HOOK32(scePthreadCondDestroy);
    // HOOK32(scePthreadKeyCreate);
    // HOOK32(scePthreadMutexattrSettype);
    // HOOK32(sceKernelDeleteEqueue);
    // HOOK32(sceKernelFstat);
    // HOOK32(sceKernelGetEventId);
    // HOOK32(scePthreadAttrInit);
    // HOOK32(scePthreadSetschedparam);
    // HOOK32(sceKernelLseek);
    // HOOK32(scePthreadJoin);
    // HOOK32(sceKernelGetDirectMemorySize);
    // HOOK32(sceKernelAllocateDirectMemory);
    // HOOK32(sceKernelVirtualQuery);
    // HOOK32(scePthreadMutexattrDestroy);
    // // HOOK32(scePthreadMutexUnlock);
    // HOOK32(scePthreadMutexTrylock);
    // // HOOK32(__tls_get_addr);
    // // HOOK32(sceKernelLoadStartModule);

    return 0;
}

s32 attr_module_hidden module_stop(s64 argc, const void *args)
{
    final_printf("[GoldHEN] <%s\\Ver.0x%08x> %s\n", g_pluginName, g_pluginVersion, __func__);
    final_printf("[GoldHEN] %s Plugin Ended.\n", g_pluginName);
    UNHOOK(sceAudioOutInit);
    return 0;
}
