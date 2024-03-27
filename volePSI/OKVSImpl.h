
#define NO_INCLUDE_PAXOS_IMPL
#include "Paxos.h"
#include <unordered_set>
#include <numeric>
#include "libOTe/Tools/LDPC/Util.h"
#include "volePSI/SimpleIndex.h"
#include <future>

namespace volePSI
{

	using AES = oc::AES;
	using PRNG = oc::PRNG;
	using SparseMtx = oc::SparseMtx;
	using DenseMtx = oc::DenseMtx;
	using BitIterator = oc::BitIterator;
	using PointList = oc::PointList;
	inline u64 OKVS::getBinSize(u64 numBins, u64 numBalls, u64 statSecParam)
	{
		return SimpleIndex::get_bin_size(numBins, numBalls, statSecParam);
	}

	template<typename ValueType>
	void OKVS::solve(span<const block> inputs, span<const ValueType> values, span<ValueType> output, PRNG* prng, u64 numThreads)
	{
		PxVector<const ValueType> V(values);
		PxVector<ValueType> P(output);
		auto h = P.defaultHelper();
		solve(inputs, V, P, prng, numThreads, h);
	}

	template<typename ValueType>
	void OKVS::solve(span<const block> inputs, MatrixView<const ValueType> values, MatrixView<ValueType> output, PRNG* prng, u64 numThreads)
	{
		if (values.cols() != output.cols())
			throw RTE_LOC;

		if (values.cols() == 1)
		{
			solve(inputs, span<const ValueType>(values), span<ValueType>(output), prng, numThreads);
		}
		else if (
			values.cols() * sizeof(ValueType) % sizeof(block) == 0 &&
			std::is_same<ValueType, block>::value == false)
		{
			auto n = values.rows();
			auto m = values.cols() * sizeof(ValueType) / sizeof(block);

			solve<block>(
				inputs,
				MatrixView<const block>((block*)values.data(), n, m),
				MatrixView<block>((block*)output.data(), output.rows(), m),
				prng,
				numThreads);
		}
		else
		{
			PxMatrix<const ValueType> V(values);
			PxMatrix<ValueType> P(output);
			auto h = P.defaultHelper();
			solve(inputs, V, P, prng, numThreads, h);
		}
	}

	template<typename Vec, typename ConstVec, typename Helper>
	void OKVS::solve(
		span<const block> inputs,
		ConstVec& V,
		Vec& P,
		PRNG* prng,
		u64 numThreads,
		Helper& h)
	{
		auto bitLength = oc::roundUpTo(oc::log2ceil((u64)(mPaxosParam.mSparseSize + 1)), 8);

		if (bitLength <= 8)
			implParSolve<u8>(inputs, V, P, prng, numThreads, h);
		else if (bitLength <= 16)
			implParSolve<u16>(inputs, V, P, prng, numThreads, h);
		else if (bitLength <= 32)
			implParSolve<u32>(inputs, V, P, prng, numThreads, h);
		else
			implParSolve<u64>(inputs, V, P, prng, numThreads, h);


		if (mDebug)
			this->check(inputs, V, P);
	}

	template<typename IdxType, typename Vec, typename ConstVec, typename Helper>
	void OKVS::implParSolve(
		span<const block> inputs_,
		ConstVec& vals_,
		Vec& p_,
		PRNG* prng,
		u64 numThreads,
		Helper& h)
	{
#ifndef NDEBUG
		{
			std::unordered_set<block> inputSet;
			for (u64 i = 0; i < inputs_.size(); ++i)
			{
				assert(inputSet.insert(inputs_[i]).second);
			}
		}
#endif
		if (p_.size() != size())
			throw RTE_LOC;

		if (mNumBins == 1)
		{
			Paxos<IdxType> paxos;
			paxos.init(mNumItems, mPaxosParam, mSeed);
			paxos.setInput(inputs_);
			paxos.encode(vals_, p_, h, prng);
			return;
		}

		numThreads = std::max<u64>(1, numThreads);

		static constexpr const u64 batchSize = 32;
		auto totalNumBins = mNumBins * numThreads;
		auto itemsPerThrd = (mNumItems + numThreads - 1) / numThreads;
		auto perThrdMaxBinSize = getBinSize(mNumBins, itemsPerThrd, mSsp);
		u64 combinedMaxBinSize = perThrdMaxBinSize * numThreads;
		Matrix<u64> thrdBinSizes(numThreads, mNumBins);
		std::unique_ptr<u64[]> inputMapping(new u64[totalNumBins * perThrdMaxBinSize]);
		auto getInputMapping = [&](u64 thrdIdx, u64 binIdx)
		{
			auto binBegin = combinedMaxBinSize * binIdx;
			auto thrdBegin = perThrdMaxBinSize * thrdIdx;
			span<u64> mapping(inputMapping.get() + binBegin + thrdBegin, perThrdMaxBinSize);
			assert(inputMapping.get() + totalNumBins * perThrdMaxBinSize >= mapping.data() + mapping.size());
			return mapping;
		};
		auto valBacking = h.newVec(totalNumBins * perThrdMaxBinSize);
		auto getValues = [&](u64 thrdIdx, u64 binIdx)
		{
			auto binBegin = combinedMaxBinSize * binIdx;
			auto thrdBegin = perThrdMaxBinSize * thrdIdx;

			return valBacking.subspan(binBegin + thrdBegin, perThrdMaxBinSize);
		};

		std::unique_ptr<block[]> hashBacking(new block[totalNumBins * perThrdMaxBinSize]);
		auto getHashes = [&](u64 thrdIdx, u64 binIdx)
		{
			auto binBegin = combinedMaxBinSize * binIdx;
			auto thrdBegin = perThrdMaxBinSize * thrdIdx;

			return span<block>(hashBacking.get() + binBegin + thrdBegin, perThrdMaxBinSize);
		};
		libdivide::libdivide_u64_t divider = libdivide::libdivide_u64_gen(mNumBins);
		AES hasher(mSeed);

		std::atomic<u64> numDone(0);
		std::promise<void> hashingDoneProm;
		auto hashingDoneFu = hashingDoneProm.get_future().share();

		auto routine = [&](u64 thrdIdx)
		{
			auto begin = (inputs_.size() * thrdIdx) / numThreads;
			auto end = (inputs_.size() * (thrdIdx + 1)) / numThreads;
			auto inputs = inputs_.subspan(begin, end - begin);
			{
				auto binSizes = thrdBinSizes[thrdIdx];
				auto inIter = inputs.data();
				std::array<block, batchSize> hashes;
				auto main = inputs.size() / batchSize * batchSize;
				std::array<u64, batchSize> binIdxs;

				u64 i = 0;
				auto inIdx = begin;
				for (; i < main;
					i += batchSize,
					inIter += batchSize)
				{
					hasher.hashBlocks<8>(inIter + 0, hashes.data() + 0);
					hasher.hashBlocks<8>(inIter + 8, hashes.data() + 8);
					hasher.hashBlocks<8>(inIter + 16, hashes.data() + 16);
					hasher.hashBlocks<8>(inIter + 24, hashes.data() + 24);
					for (u64 k = 0; k < batchSize; ++k)
						binIdxs[k] = binIdxCompress(hashes[k]);

					doMod32(binIdxs.data(), &divider, mNumBins);

					for (u64 k = 0; k < batchSize; ++k, ++inIdx)
					{
						auto binIdx = binIdxs[k];
						auto bs = binSizes[binIdx]++;
						getInputMapping(thrdIdx, binIdx)[bs] = inIdx;
						h.assign(getValues(thrdIdx, binIdx)[bs], vals_[inIdx]);
						getHashes(thrdIdx, binIdx)[bs] = hashes[k];
					}
				}
				for (u64 k = 0; i < inputs.size(); ++i, ++inIter, ++k, ++inIdx)
				{
					hashes[k] = hasher.hashBlock(*inIter);

					auto binIdx = modNumBins(hashes[k]);
					auto bs = binSizes[binIdx]++;
					assert(bs < perThrdMaxBinSize);

					if (inIdx == 9355778)
						std::cout << "in " << inIdx << " -> bin " << binIdx << " @ " << bs << std::endl;
					getInputMapping(thrdIdx, binIdx)[bs] = inIdx;
					h.assign(getValues(thrdIdx, binIdx)[bs], vals_[inIdx]);
					getHashes(thrdIdx, binIdx)[bs] = hashes[k];
				}
			}
			auto paxosSizePer = mPaxosParam.size();
			auto allocSize =
				sizeof(IdxType) * (
					mItemsPerBin * mWeight * 2 +
					mPaxosParam.mSparseSize
					) +
				sizeof(span<IdxType>) * mPaxosParam.mSparseSize;
			std::unique_ptr<u8[]> allocation(new u8[allocSize]);
			if (++numDone == numThreads)
				hashingDoneProm.set_value();
			else
				hashingDoneFu.get();

			Paxos<IdxType> paxos;
			for (u64 binIdx = thrdIdx; binIdx < mNumBins; binIdx += numThreads)
			{
				u64 binSize = 0;
				for (u64 i = 0; i < numThreads; ++i)
					binSize += thrdBinSizes(i, binIdx);

				if (binSize > mItemsPerBin)
					throw RTE_LOC;

				paxos.init(binSize, mPaxosParam, mSeed);

				auto iter = allocation.get();
				MatrixView<IdxType> rows = initMV<IdxType>(iter, binSize, mWeight);
				span<IdxType> colBacking = initSpan<IdxType>(iter, binSize * mWeight);
				span<IdxType> colWeights = initSpan<IdxType>(iter, mPaxosParam.mSparseSize);
				span<span<IdxType>> cols = initSpan<span<IdxType>>(iter, mPaxosParam.mSparseSize);

				if (iter > allocation.get() + allocSize)
					throw RTE_LOC;

				auto binBegin = combinedMaxBinSize * binIdx;
				auto values = valBacking.subspan(binBegin, binSize);
				auto hashes = span<block>(hashBacking.get() + binBegin, binSize);
				auto output = p_.subspan(paxosSizePer * binIdx, paxosSizePer);
				u64 binPos = thrdBinSizes(0, binIdx);
				assert(binPos <= perThrdMaxBinSize);
				assert(hashes.data() == getHashes(0, binIdx).data());

				for (u64 i = 1; i < numThreads; ++i)
				{
					auto size = thrdBinSizes(i, binIdx);
					assert(size <= perThrdMaxBinSize);
					auto thrdHashes = getHashes(i, binIdx);
					auto thrdVals = getValues(i, binIdx);
					memmove(hashes.data() + binPos, thrdHashes.data(), size * sizeof(block));
					for (u64 j = 0; j < size; ++j)
						h.assign(values[binPos + j], thrdVals[j]);

					binPos += size;
				}
				std::memset(colWeights.data(), 0, colWeights.size() * sizeof(IdxType));
				auto rIter = rows.data();
				if (mWeight == 3)
				{
					auto main = binSize / batchSize * batchSize;

					u64 i = 0;
					for (; i < main; i += batchSize)
					{
						paxos.mHasher.buildRow32(&hashes[i], rIter);
						for (u64 j = 0; j < batchSize; ++j)
						{
							++colWeights[rIter[0]];
							++colWeights[rIter[1]];
							++colWeights[rIter[2]];
							rIter += mWeight;
						}
					}
					for (; i < binSize; ++i)
					{
						paxos.mHasher.buildRow(hashes[i], rIter);

						++colWeights[rIter[0]];
						++colWeights[rIter[1]];
						++colWeights[rIter[2]];
						rIter += mWeight;
					}
				}
				else
				{
					for (u64 i = 0; i < binSize; ++i)
					{
						paxos.mHasher.buildRow(hashes[i], rIter);
						for (u64 k = 0; k < mWeight; ++k)
							++colWeights[rIter[k]];
						rIter += mWeight;
					}
				}

				paxos.setInput(rows, hashes, cols, colBacking, colWeights);
				paxos.encode(values, output, h, prng);

			}
		};

		std::vector<std::thread> thrds(numThreads - 1);

		for (u64 i = 0; i < thrds.size(); ++i)
			thrds[i] = std::thread(routine, i);

		routine(thrds.size());

		for (u64 i = 0; i < thrds.size(); ++i)
			thrds[i].join();
	}

	template<typename ValueType>
	void OKVS::decode(span<const block> inputs, span<ValueType> values, span<const ValueType> p, u64 numThreads)
	{
		PxVector<ValueType> V(values);
		PxVector<const ValueType> P(p);
		auto h = V.defaultHelper();

		decode(inputs, V, P, h, numThreads);
	}


	template<typename ValueType>
	void OKVS::decode(span<const block> inputs, MatrixView<ValueType> values, MatrixView<const ValueType> p, u64 numThreads)
	{

		if (values.cols() != p.cols())
			throw RTE_LOC;

		if (values.cols() == 1)
		{
			decode(inputs, span<ValueType>(values), span<const ValueType>(p), numThreads);
		}
		else if (
			values.cols() * sizeof(ValueType) % sizeof(block) == 0 &&
			std::is_same<ValueType, block>::value == false)
		{
			auto n = values.rows();
			auto m = values.cols() * sizeof(ValueType) / sizeof(block);

			decode<block>(
				inputs,
				MatrixView<block>((block*)values.data(), n, m),
				MatrixView<const block>((block*)p.data(), p.rows(), m));
		}
		else
		{
			PxMatrix<ValueType> V(values);
			PxMatrix<const ValueType> P(p);
			auto h = V.defaultHelper();

			decode(inputs, V, P, h, numThreads);
		}
	}

	template<typename Vec, typename ConstVec, typename Helper>
	void OKVS::decode(
		span<const block> inputs,
		Vec& V,
		ConstVec& P,
		Helper& h,
		u64 numThreads)
	{
		auto bitLength = oc::roundUpTo(oc::log2ceil((u64)(mPaxosParam.mSparseSize + 1)), 8);
		if (bitLength <= 8)
			implParDecode<u8>(inputs, V, P, h, numThreads);
		else if (bitLength <= 16)
			implParDecode<u16>(inputs, V, P, h, numThreads);
		else if (bitLength <= 32)
			implParDecode<u32>(inputs, V, P, h, numThreads);
		else
			implParDecode<u64>(inputs, V, P, h, numThreads);
	}


	template<typename IdxType, typename Vec, typename ConstVec, typename Helper>
	void OKVS::implDecodeBin(
		u64 binIdx,
		span<block> hashes,
		Vec& values,
		Vec& valuesBuff,
		span<u64> inIdxs,
		ConstVec& PP,
		Helper& h,
		Paxos<IdxType>& paxos)
	{
		constexpr u64 batchSize = 32;
		constexpr u64 maxWeightSize = 20;

		auto main = (hashes.size() / batchSize) * batchSize;

		assert(mWeight <= maxWeightSize);
		std::array<IdxType, maxWeightSize* batchSize> _backing;
		MatrixView<IdxType> row(_backing.data(), batchSize, mWeight);
		assert(valuesBuff.size() >= batchSize);
		u64 i = 0;
		for (; i < main; i += batchSize)
		{
			paxos.mHasher.buildRow32(&hashes[i], row.data());
			paxos.decode32(row.data(), &hashes[i], valuesBuff[0], PP, h);


			if (mAddToDecode)
			{
				for (u64 k = 0; k < batchSize; ++k)
					h.add(values[inIdxs[i + k]], valuesBuff[k]);
			}
			else
			{
				for (u64 k = 0; k < batchSize; ++k)
					h.assign(values[inIdxs[i + k]], valuesBuff[k]);
			}
		}

		for (; i < hashes.size(); ++i)
		{
			paxos.mHasher.buildRow(hashes[i], row.data());
			auto v = values[inIdxs[i]];

			if (mAddToDecode)
			{
				paxos.decode1(row.data(), &hashes[i], valuesBuff[0], PP, h);
				h.add(v, valuesBuff[0]);
			}
			else
				paxos.decode1(row.data(), &hashes[i], v, PP, h);
		}
	}


	template<typename IdxType, typename Vec, typename ConstVec, typename Helper>
	void OKVS::implDecodeBatch(span<const block> inputs, Vec& values, ConstVec& pp, Helper& h)
	{
		u64 decodeSize = std::min<u64>(512, inputs.size());
		Matrix<block> batches(mNumBins, decodeSize);
		Matrix<u64> inIdxs(mNumBins, decodeSize);
		std::vector<u64> batchSizes(mNumBins);

		AES hasher(mSeed);
		auto inIter = inputs.data();
		Paxos<IdxType> paxos;
		auto sizePer = size() / mNumBins;
		paxos.init(1, mPaxosParam, mSeed);
		auto buff = h.newVec(32);
		static const u32 batchSize = 32;
		auto main = inputs.size() / batchSize * batchSize;
		std::array<block, batchSize> buffer;
		std::array<u64, batchSize> binIdxs;
		u64 i = 0;
		libdivide::libdivide_u64_t divider = libdivide::libdivide_u64_gen(mNumBins);
		for (; i < main; i += batchSize, inIter += batchSize)
		{
			hasher.hashBlocks<8>(inIter, buffer.data());
			hasher.hashBlocks<8>(inIter + 8, buffer.data() + 8);
			hasher.hashBlocks<8>(inIter + 16, buffer.data() + 16);
			hasher.hashBlocks<8>(inIter + 24, buffer.data() + 24);

			for (u64 j = 0; j < batchSize; j += 8)
			{
				binIdxs[j + 0] = binIdxCompress(buffer[j + 0]);
				binIdxs[j + 1] = binIdxCompress(buffer[j + 1]);
				binIdxs[j + 2] = binIdxCompress(buffer[j + 2]);
				binIdxs[j + 3] = binIdxCompress(buffer[j + 3]);
				binIdxs[j + 4] = binIdxCompress(buffer[j + 4]);
				binIdxs[j + 5] = binIdxCompress(buffer[j + 5]);
				binIdxs[j + 6] = binIdxCompress(buffer[j + 6]);
				binIdxs[j + 7] = binIdxCompress(buffer[j + 7]);
			}

			doMod32(binIdxs.data(), &divider, mNumBins);

			for (u64 k = 0; k < batchSize; ++k)
			{
				auto binIdx = binIdxs[k];

				batches(binIdx, batchSizes[binIdx]) = buffer[k];
				inIdxs(binIdx, batchSizes[binIdx]) = i + k;
				++batchSizes[binIdx];

				if (batchSizes[binIdx] == decodeSize)
				{
					auto p = pp.subspan(binIdx * sizePer, sizePer);
					auto idxs = inIdxs[binIdx];
					implDecodeBin(binIdx, batches[binIdx], values, buff, idxs, p, h, paxos);

					batchSizes[binIdx] = 0;
				}
			}
		}

		for (; i < inputs.size(); ++i, ++inIter)
		{
			auto k = 0;
			buffer[k] = hasher.hashBlock(*inIter);
			auto binIdx = modNumBins(buffer[k]);

			batches(binIdx, batchSizes[binIdx]) = buffer[k];
			inIdxs(binIdx, batchSizes[binIdx]) = i + k;
			++batchSizes[binIdx];
			if (batchSizes[binIdx] == decodeSize)
			{
				auto p = pp.subspan(binIdx * sizePer, sizePer);
				implDecodeBin(binIdx, batches[binIdx], values, buff, inIdxs[binIdx], p, h, paxos);

				batchSizes[binIdx] = 0;
			}
		}

		for (u64 binIdx = 0; binIdx < mNumBins; ++binIdx)
		{
			if (batchSizes[binIdx])
			{
				auto p = pp.subspan(binIdx * sizePer, sizePer);
				auto b = batches[binIdx].subspan(0, batchSizes[binIdx]);
				implDecodeBin(binIdx, b, values, buff, inIdxs[binIdx], p, h, paxos);
			}
		}
	}
	template<typename IdxType, typename Vec, typename ConstVec, typename Helper>
	void OKVS::implParDecode(
		span<const block> inputs,
		Vec& values,
		ConstVec& pp,
		Helper& h,
		u64 numThreads)
	{
		if (mNumBins == 1)
		{
			Paxos<IdxType> paxos;
			paxos.init(1, mPaxosParam, mSeed);
			paxos.mAddToDecode = mAddToDecode;
			paxos.decode(inputs, values, pp, h);
			return;
		}
		numThreads = std::max<u64>(numThreads, 1ull);

		std::vector<std::thread> thrds(numThreads - 1);
		auto routine = [&](u64 i)
		{
			auto begin = (inputs.size() * i) / numThreads;
			auto end = (inputs.size() * (i + 1)) / numThreads;
			span<const block> in(inputs.begin() + begin, inputs.begin() + end);
			auto va = values.subspan(begin, end - begin);
			implDecodeBatch<IdxType>(in, va, pp, h);
		};

		for (u64 i = 0; i < thrds.size(); ++i)
			thrds[i] = std::thread(routine, i);

		routine(thrds.size());

		for (u64 i = 0; i < thrds.size(); ++i)
			thrds[i].join();
	}
}
