#pragma once
// © 2019 Peter Rindal.
// © 2022 Visa.
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
// 
// The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
// 
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

#include <libOTe/config.h>
#ifdef ENABLE_MR_KYBER

#include "libOTe/TwoChooseOne/OTExtInterface.h"
#include <cryptoTools/Common/Defines.h>
#include <cryptoTools/Crypto/PRNG.h>
#include "libOTe/Tools/Coproto.h"
extern "C"
{
#include <KyberOT/KyberOT.h>
}

namespace osuCrypto
{


    class MasnyRindalKyber : public OtReceiver, public OtSender
    {
    public:

        task<> receive(
            const BitVector& choices,
            span<block> messages,
            PRNG& prng,
            Socket& chl,
            u64 numThreads)
        {
            return receive(choices, messages, prng, chl);
        }

        task<> send(
            span<std::array<block, 2>> messages,
            PRNG& prng,
            Socket& chl,
            u64 numThreads)
        {
            return send(messages, prng, chl);
        }

        task<> receive(
            const BitVector& choices,
            span<block> messages,
            PRNG& prng,
            Socket& chl) override;

        task<> send(
            span<std::array<block, 2>> messages,
            PRNG& prng,
            Socket& chl) override;
    };


}
#endif