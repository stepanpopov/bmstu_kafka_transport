SPLIT_CODE_SERVICE_URL ?= http://localhost:8081/code
CONSUME_RECEIVE_SERVICE_URL ?= http://localhost:8082/receive
CHUNK_BYTE_SIZE ?= 200

BROKERS ?= localhost:9094
TOPIC ?= test
COUNSUMER_GROUP_ID ?= aaa

TEST_SERVER_PORT ?= 8082

# cd transport && docker build -t consume --target consume . // for single

build-docker:
	docker-compose build .

run-split:
	cd transport && cargo run --bin split -- --code-service-url=${SPLIT_CODE_SERVICE_URL} --chunk_byte_size=${CHUNK_BYTE_SIZE}

run-produce:
	cd transport && cargo run --bin produce -- --brokers=${BROKERS} --topic=${TOPIC}

run-consume:
	cd transport && cargo run --bin consume -- --brokers=${BROKERS} --topic=${TOPIC} --group-id=${COUNSUMER_GROUP_ID} --receive_url=${CONSUME_RECEIVE_SERVICE_URL}

run-test-code-server:
	python3 ./test_server.py 8081

run-test-receive-server:
	python3 ./test_server.py ${TEST_SERVER_PORT}
