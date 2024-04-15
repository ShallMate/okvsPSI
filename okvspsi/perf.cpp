#include "perf.h"
#include "cryptoTools/Network/IOService.h"
#include "volePSI/OkvsPsi.h"
#include "volePSI/SimpleIndex.h"
#include "coproto/Socket/AsioSocket.h"
#include "libdivide.h"
#include "volePSI/fileBased.h"

using namespace oc;
using namespace volePSI;




void perfOkvsPSI(oc::CLP& cmd)
{
	//auto n = 1ull << cmd.getOr("nn", 10);
	auto ns = 1ull << cmd.getOr("nns", 10);
	auto nr = 1ull << cmd.getOr("nnr", 10);
	auto t = cmd.getOr("t", 1ull);
	auto e = cmd.getOr("e", 0.01);
	auto mal = cmd.isSet("m");
	auto v = cmd.isSet("v") ? cmd.getOr("v", 1) : 0;
	auto nt = cmd.getOr("nt", 1);
	bool fakeBase = cmd.isSet("f");
	bool noCompress = cmd.isSet("nc");
	auto type = oc::DefaultMultType;
	//auto type = oc::MultType::QuasiCyclic;
	PRNG prng(ZeroBlock);
	Timer timer, s, r;
	std::cout << "thread = " << nt << std::endl;
	std::cout<< "The sender input size = " << ns << std::endl;
	std::cout<<"The receiver input size = "<<nr<<std::endl;
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
	recv.init(ns, nr, 40, ZeroBlock, mal, nt);
	send.init(ns, nr, 40, ZeroBlock, mal, nt);

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

	std::vector<block> recvSet(nr), sendSet(ns);
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
		std::cout <<"The receiver sends "<< bytesSent(sockets,0,e) << " bytes." <<std::endl;
		std::cout<<"The sender sends " <<bytesSent(sockets,1,e)<<" bytes." << std::endl;
		std::cout<<"The communication overhead = " <<double(bytesSent(sockets,0,e)+bytesSent(sockets,1,e))/1024/1024<<" MB." << std::endl;
		if (v > 1)
			std::cout << "s\n" << s << "\nr\n" << r << std::endl;
		//std::cout <<"-------------log--------------------\n" << coproto::getLog() << std::endl;
	}
}

void networkSocketExampleRun(oc::CLP& cmd)
{
    try {
        auto recver = cmd.get<int>("r");
        bool client = !cmd.getOr("server", recver);
        auto ip = cmd.getOr<std::string>("ip", "localhost:1212");
        auto ns = cmd.getOr("ns", 100ull);
        auto nr = cmd.getOr("nr", 100ull);
        // The statistical security parameter.
        auto ssp = cmd.getOr("ssp", 40ull);
        // Malicious Security.
        auto mal = cmd.isSet("malicious");
        // The vole type, default to expand accumulate.
        auto type = oc::DefaultMultType;
        // use fewer rounds of communication but more computation.
        auto useReducedRounds = cmd.isSet("reducedRounds");
        std::cout << "connecting as " << (client ? "client" : "server") << " at ip " << ip << std::endl;
        coproto::Socket sock;
#ifdef COPROTO_ENABLE_BOOST
            // Perform the TCP/IP.
            sock = coproto::asioConnect(ip, !client);
#else
            throw std::runtime_error("COPROTO_ENABLE_BOOST must be define (via cmake) to use tcp sockets. " COPROTO_LOCATION);
#endif
        std::cout << "connected" << std::endl;
        std::vector<oc::block> set;
        if (!recver)
        {
            // Use dummy set {0,1,...}
            set.resize(ns);
            for (oc::u64 i = 0; i < ns; ++i)
                set[i] = oc::block(0, i);

            // configure
            volePSI::OkvsPsiSender sender;
            sender.setMultType(type);
            sender.init(ns, nr, ssp, oc::sysRandomSeed(), mal, 1, useReducedRounds);

            std::cout << "sender start\n";
            auto start = std::chrono::system_clock::now();

            // Run the protocol.
            macoro::sync_wait(sender.run(set, sock));

            auto done = std::chrono::system_clock::now();
            std::cout << "sender done, " << std::chrono::duration_cast<std::chrono::milliseconds>(done-start).count() <<"ms" << std::endl;
        }
        else
        {
            // Use dummy set {0,1,...}
            set.resize(nr);
            for (oc::u64 i = 0; i < nr; ++i)
                set[i] = oc::block(0, i);

            // Configure.
            volePSI::OkvsPsiReceiver recevier;
            recevier.setMultType(type);
            recevier.init(ns, nr, ssp, oc::sysRandomSeed(), mal, 1, useReducedRounds);

            std::cout << "recver start\n";
            auto start = std::chrono::system_clock::now();

            // Run the protocol.
            macoro::sync_wait(recevier.run(set, sock));

            auto done = std::chrono::system_clock::now();
            std::cout << "sender done, " << std::chrono::duration_cast<std::chrono::milliseconds>(done-start).count() <<"ms" << std::endl;
        }
    }
    catch (std::exception& e)
    {
        std::cout << e.what() << std::endl;
    }
}








