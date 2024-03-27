#pragma once
// © 2022 Visa.
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.


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
}