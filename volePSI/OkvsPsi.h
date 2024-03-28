#pragma once
#include "volePSI/Defines.h"
#include "volePSI/OkvsOprf.h"
#include "sparsehash/dense_hash_map"
#include "cryptoTools/Common/Timer.h"

namespace volePSI
{
    namespace details
    {
        struct OkvsPsiBase
        {

            u64 mSenderSize = 0;
            u64 mRecverSize = 0;
            u64 mSsp = 0;
            PRNG mPrng;
            bool mMalicious = false;
            bool mCompress = true;
            u64 mNumThreads = 0;
            u64 mMaskSize = 0;
            bool mUseReducedRounds = false;
            bool mDebug = false;

            void init(u64 senderSize, u64 recverSize, u64 statSecParam, block seed, bool malicious, u64 numThreads, bool useReducedRounds = false);

        };
    }

    class OkvsPsiSender : public details::OkvsPsiBase, public oc::TimerAdapter
    {
    public:

        OkvsOprfSender mSender;
        void setMultType(oc::MultType type) { mSender.setMultType(type); };


        Proto run(span<block> inputs, Socket& chl);
    };


    class OkvsPsiReceiver : public details::OkvsPsiBase, public oc::TimerAdapter
    {
    public:
        OkvsOprfReceiver mRecver;
        void setMultType(oc::MultType type) { mRecver.setMultType(type); };

        std::vector<u64> mIntersection;

        Proto run(span<block> inputs, Socket& chl);
    };

    std::size_t bytesSent(std::array<osuCrypto::cp::LocalAsyncSocket, 2> sockets,int role,auto e) {
			u64 com = sockets[role].bytesSent();
            
            if (role ==0){
                com = (1+e)/1.3*com;
            }
			return com;
		}
}