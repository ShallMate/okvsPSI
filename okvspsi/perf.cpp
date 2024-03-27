#include "perf.h"
#include "cryptoTools/Network/IOService.h"
#include "volePSI/OkvsPsi.h"
#include "volePSI/SimpleIndex.h"

#include "libdivide.h"
using namespace oc;
using namespace volePSI;;

void perfBaxos(oc::CLP& cmd)
{
	auto n = cmd.getOr("n", 1ull << cmd.getOr("nn", 10));
	auto t = cmd.getOr("t", 1ull);
	//auto rand = cmd.isSet("rand");
	auto v = cmd.getOr("v", cmd.isSet("v") ? 1 : 0);
	auto w = cmd.getOr("w", 3);
	auto ssp = cmd.getOr("ssp", 40);
	auto dt = cmd.isSet("binary") ? PaxosParam::Binary : PaxosParam::GF128;
	auto nt = cmd.getOr("nt", 0);

	//PaxosParam pp(n, w, ssp, dt);
	auto binSize = 1 << cmd.getOr("lbs", 15);
	u64 baxosSize;
	{
		Baxos paxos;
		paxos.init(n, binSize, w, ssp, dt, oc::ZeroBlock);
		baxosSize = paxos.size();
	}
	std::vector<block> key(n), val(n), pax(baxosSize);
	PRNG prng(ZeroBlock);
	prng.get<block>(key);
	prng.get<block>(val);

	Timer timer;
	auto start = timer.setTimePoint("start");
	auto end = start;
	for (u64 i = 0; i < t; ++i)
	{
		Baxos paxos;
		paxos.init(n, binSize, w, ssp, dt, block(i, i));

		//if (v > 1)
		//	paxos.setTimer(timer);

		paxos.solve<block>(key, val, pax, nullptr, nt);
		timer.setTimePoint("s" + std::to_string(i));

		paxos.decode<block>(key, val, pax, nt);

		end = timer.setTimePoint("d" + std::to_string(i));
	}

	if (v)
		std::cout << timer << std::endl;

	auto tt = std::chrono::duration_cast<std::chrono::microseconds>(end - start).count() / double(1000);
	std::cout << "total " << tt << "ms, e=" << double(baxosSize) / n << std::endl;
}



void perfPSI(oc::CLP& cmd)
{
	auto n = 1ull << cmd.getOr("nn", 10);
	auto t = cmd.getOr("t", 1ull);
	auto mal = cmd.isSet("malicious");
	auto v = cmd.isSet("v") ? cmd.getOr("v", 1) : 0;
	auto nt = cmd.getOr("nt", 1);
	bool fakeBase = cmd.isSet("fakeBase");
	bool noCompress = cmd.isSet("nc");
	auto type = oc::DefaultMultType;
	PRNG prng(ZeroBlock);
	Timer timer, s, r;
	std::cout << "nt " << nt << " fakeBase " << int(fakeBase) << " n " << n << std::endl;
	OkvsPsiReceiver recv;
	OkvsPsiSender send;

	if (fakeBase)
	{
		std::vector<std::array<block, 2>> sendBase(128);
		std::vector<block> recvBase(128);
		BitVector recvChoice(128);
		recvChoice.randomize(prng);
		prng.get(sendBase.data(), sendBase.size());
		for (u64 i = 0; i < 128; ++i)
			recvBase[i] = sendBase[i][recvChoice[i]];
		recv.mRecver.mVoleRecver.setBaseOts(sendBase);
		send.mSender.mVoleSender.setBaseOts(recvBase, recvChoice);
		timer.setTimePoint("fakeBase");
	}
	recv.init(n, n, 40, ZeroBlock, mal, nt);
	send.init(n, n, 40, ZeroBlock, mal, nt);

	recv.setMultType(type);
	send.setMultType(type);

	if (noCompress)
	{
		recv.mCompress = false;
		send.mCompress = false;
		recv.mMaskSize = sizeof(block);
		send.mMaskSize = sizeof(block);
	}


	if (cmd.hasValue("bs") || cmd.hasValue("lbs"))
	{
		u64 binSize = cmd.getOr("bs", 1ull << cmd.getOr("lbs", 15));
		recv.mRecver.mBinSize = binSize;
		send.mSender.mBinSize = binSize;
	}

	std::vector<block> recvSet(n), sendSet(n);
	prng.get<block>(recvSet);
	prng.get<block>(sendSet);

	recv.setTimer(r);
	send.setTimer(s);

	auto sockets = cp::LocalAsyncSocket::makePair();

	for (u64 i = 0; i < t; ++i)
	{
		auto p0 = recv.run(recvSet, sockets[0]);
		auto p1 = send.run(sendSet, sockets[1]);
		s.setTimePoint("begin");
		r.setTimePoint("begin");
		timer.setTimePoint("begin");
		auto r = macoro::sync_wait(macoro::when_all_ready(std::move(p0), std::move(p1)));
		try{ std::get<0>(r).result(); } catch(std::exception& e) {std::cout << e.what() << std::endl; }
		try{ std::get<1>(r).result(); } catch(std::exception& e) {std::cout << e.what() << std::endl; }
		timer.setTimePoint("end");

	}
	if (v)
	{

		std::cout << timer << std::endl;
		std::cout << sockets[0].bytesSent() << " " << sockets[1].bytesSent() << std::endl;
		if (v > 1)
			std::cout << "s\n" << s << "\nr\n" << r << std::endl;
		//std::cout <<"-------------log--------------------\n" << coproto::getLog() << std::endl;
	}
}





