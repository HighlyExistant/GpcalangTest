# importing the required module
import matplotlib.pyplot as plt

f = open("log.txt");
title = f.readline();
buf = f.read();
split_buf = buf.split();
print(split_buf);

# Seed {} Mutation {} UseEnergy? {} Width {} Height {}
# x axis values
x = []
# corresponding y axis values
y = []
buf_range = range(0, len(split_buf), 2);
for i in buf_range:
    if split_buf[i] == "Frame":
        x.append(int(split_buf[i+1].replace(",", ' ')));
    if split_buf[i] == "EntityCount:":
        y.append(int(split_buf[i+1].replace(",", ' ')));
# plotting the points 
plt.plot(x, y)
# naming the x axis
plt.xlabel('Frames')
# naming the y axis
plt.ylabel('Entity Count')
# giving a title to my graph
plt.title(title,fontsize=8)
# function to show the plot
plt.show()