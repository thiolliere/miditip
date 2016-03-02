device=$1
delay=$2
deviation=$3

echo "sudo tc qdisc add dev $device root netem delay $delay $deviation distribution normal"
sudo tc qdisc add dev $device root netem delay $delay $deviation distribution normal

echo "press enter when done"
read

echo "sudo tc qdisc del dev $device root"
sudo tc qdisc del dev $device root


