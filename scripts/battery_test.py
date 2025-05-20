#!/usr/bin/env python3
import subprocess
import time
import datetime
import csv
import matplotlib.pyplot as plt
import os
import signal
import sys

class BatteryTest:
    def __init__(self):
        self.data = []
        self.is_acs_running = False
        self.output_dir = "battery_test_results"
        os.makedirs(self.output_dir, exist_ok=True)

    def get_battery_info(self):
        try:
            output = subprocess.check_output("acs get power --raw", shell=True).decode()
            lid, bat, ac = output.split()
            return int(bat), ac == "true"
        except:
            print("Error getting battery info")
            return None, None

    def log_data(self, message=""):
        bat, ac = self.get_battery_info()
        if bat is None:
            return

        timestamp = datetime.datetime.now()
        self.data.append({
            'timestamp': timestamp,
            'battery': bat,
            'is_acs_running': self.is_acs_running,
            'is_charging': ac,
            'message': message
        })

        print(f"[{timestamp}] Battery: {bat}% (ACS: {self.is_acs_running}, Charging: {ac}) - {message}")

    def save_data(self, filename):
        with open(filename, 'w', newline='') as f:
            writer = csv.DictWriter(f, fieldnames=['timestamp', 'battery', 'is_acs_running', 'is_charging', 'message'])
            writer.writeheader()
            writer.writerows(self.data)

    def plot_results(self, filename):
        plt.figure(figsize=(12, 6))

        # Plot ACS running data
        acs_data = [d for d in self.data if d['is_acs_running']]
        if acs_data:
            times = [d['timestamp'] for d in acs_data]
            battery = [d['battery'] for d in acs_data]
            plt.plot(times, battery, label='ACS Running', color='red')

        # Plot ACS not running data
        no_acs_data = [d for d in self.data if not d['is_acs_running']]
        if no_acs_data:
            times = [d['timestamp'] for d in no_acs_data]
            battery = [d['battery'] for d in no_acs_data]
            plt.plot(times, battery, label='ACS Not Running', color='blue')

        plt.title('Battery Drain Test')
        plt.xlabel('Time')
        plt.ylabel('Battery Percentage')
        plt.grid(True)
        plt.legend()
        plt.xticks(rotation=45)
        plt.tight_layout()
        plt.savefig(filename)
        plt.close()

    def run_test(self, with_acs=True):
        self.is_acs_running = with_acs
        self.data = []  # Clear previous data

        # Start ACS if needed
        if with_acs:
            subprocess.Popen(["sudo", "systemctl", "start", "acs"])
            time.sleep(5)  # Wait for ACS to start
        else:
            subprocess.Popen(["sudo", "systemctl", "stop", "acs"])
            time.sleep(5)  # Wait for ACS to stop

        self.log_data("Test started")

        # Wait for battery to reach 95%
        while True:
            bat, ac = self.get_battery_info()
            if bat is None:
                continue

            if bat <= 95 and not ac:
                break

            print(f"Waiting for battery to reach 95%... Current: {bat}%")
            time.sleep(60)

        # Record data until battery reaches 5%
        while True:
            bat, ac = self.get_battery_info()
            if bat is None:
                continue

            self.log_data()

            if bat <= 5:
                self.log_data("Test ended")
                break

            time.sleep(60)

        # Save results
        timestamp = datetime.datetime.now().strftime("%Y%m%d_%H%M%S")
        self.save_data(f"{self.output_dir}/battery_test_{timestamp}_{'with' if with_acs else 'without'}_acs.csv")
        self.plot_results(f"{self.output_dir}/battery_test_{timestamp}_{'with' if with_acs else 'without'}_acs.png")

def signal_handler(sig, frame):
    print("\nTest interrupted. Cleaning up...")
    subprocess.run(["sudo", "systemctl", "stop", "acs"])
    sys.exit(0)

if __name__ == "__main__":
    signal.signal(signal.SIGINT, signal_handler)

    test = BatteryTest()

    print("Starting battery test with ACS...")
    test.run_test(with_acs=True)

    print("\nWaiting for battery to charge back to 95%...")
    while True:
        bat, ac = test.get_battery_info()
        if bat is None:
            continue

        if bat >= 95 and ac:
            break

        print(f"Current battery: {bat}%")
        time.sleep(60)

    print("\nStarting battery test without ACS...")
    test.run_test(with_acs=False)

    print("\nTest complete! Results saved in battery_test_results directory.")
